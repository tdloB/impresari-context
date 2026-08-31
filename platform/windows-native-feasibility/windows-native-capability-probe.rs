// SPDX-License-Identifier: Apache-2.0
#![cfg(windows)]
#![deny(unsafe_op_in_unsafe_fn)]
#![allow(non_snake_case)]

use std::{
    env,
    ffi::{OsStr, c_void},
    mem::{size_of, zeroed},
    os::windows::ffi::OsStrExt,
    path::Path,
    ptr::{null, null_mut},
};

const PROFILE_ID: &str = "iar-windows-native-feasibility-v1";
const PROFILE_DIGEST: &str =
    "sha256:6b8f614387fc97321497e6b725213b9ee3c2159f3d1384fb800ffbe8af490a73";
const JOB_OBJECT_EXTENDED_LIMIT_INFORMATION_CLASS: i32 = 9;
const JOB_OBJECT_LIMIT_ACTIVE_PROCESS: u32 = 0x0000_0008;
const JOB_OBJECT_LIMIT_PROCESS_MEMORY: u32 = 0x0000_0100;
const JOB_OBJECT_LIMIT_JOB_MEMORY: u32 = 0x0000_0200;
const JOB_OBJECT_LIMIT_BREAKAWAY_OK: u32 = 0x0000_0800;
const JOB_OBJECT_LIMIT_SILENT_BREAKAWAY_OK: u32 = 0x0000_1000;
const JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE: u32 = 0x0000_2000;

type Handle = *mut c_void;
type Sid = *mut c_void;

#[repr(C)]
#[derive(Clone, Copy)]
struct OsVersionInfoW {
    dwOSVersionInfoSize: u32,
    dwMajorVersion: u32,
    dwMinorVersion: u32,
    dwBuildNumber: u32,
    dwPlatformId: u32,
    szCSDVersion: [u16; 128],
}

#[repr(C)]
#[derive(Clone, Copy, Default)]
struct JobObjectBasicLimitInformation {
    PerProcessUserTimeLimit: i64,
    PerJobUserTimeLimit: i64,
    LimitFlags: u32,
    MinimumWorkingSetSize: usize,
    MaximumWorkingSetSize: usize,
    ActiveProcessLimit: u32,
    Affinity: usize,
    PriorityClass: u32,
    SchedulingClass: u32,
}

#[repr(C)]
#[derive(Clone, Copy, Default)]
struct IoCounters {
    ReadOperationCount: u64,
    WriteOperationCount: u64,
    OtherOperationCount: u64,
    ReadTransferCount: u64,
    WriteTransferCount: u64,
    OtherTransferCount: u64,
}

#[repr(C)]
#[derive(Clone, Copy, Default)]
struct JobObjectExtendedLimitInformation {
    BasicLimitInformation: JobObjectBasicLimitInformation,
    IoInfo: IoCounters,
    ProcessMemoryLimit: usize,
    JobMemoryLimit: usize,
    PeakProcessMemoryUsed: usize,
    PeakJobMemoryUsed: usize,
}

#[link(name = "kernel32")]
unsafe extern "system" {
    fn CreateJobObjectW(attributes: *const c_void, name: *const u16) -> Handle;
    fn SetInformationJobObject(
        job: Handle,
        information_class: i32,
        information: *const c_void,
        information_length: u32,
    ) -> i32;
    fn QueryInformationJobObject(
        job: Handle,
        information_class: i32,
        information: *mut c_void,
        information_length: u32,
        returned_length: *mut u32,
    ) -> i32;
    fn CloseHandle(handle: Handle) -> i32;
    fn GetCurrentProcessId() -> u32;
    fn GetTickCount64() -> u64;
    fn GetModuleHandleW(module_name: *const u16) -> Handle;
    fn GetProcAddress(module: Handle, procedure_name: *const u8) -> *mut c_void;
    fn GetVolumeInformationW(
        root_path_name: *const u16,
        volume_name_buffer: *mut u16,
        volume_name_size: u32,
        volume_serial_number: *mut u32,
        maximum_component_length: *mut u32,
        filesystem_flags: *mut u32,
        filesystem_name_buffer: *mut u16,
        filesystem_name_size: u32,
    ) -> i32;
}

#[link(name = "ntdll")]
unsafe extern "system" {
    fn RtlGetVersion(version: *mut OsVersionInfoW) -> i32;
}

#[link(name = "userenv")]
unsafe extern "system" {
    fn CreateAppContainerProfile(
        appcontainer_name: *const u16,
        display_name: *const u16,
        description: *const u16,
        capabilities: *const c_void,
        capability_count: u32,
        appcontainer_sid: *mut Sid,
    ) -> i32;
    fn DeriveAppContainerSidFromAppContainerName(
        appcontainer_name: *const u16,
        appcontainer_sid: *mut Sid,
    ) -> i32;
    fn DeleteAppContainerProfile(appcontainer_name: *const u16) -> i32;
}

#[link(name = "advapi32")]
unsafe extern "system" {
    fn EqualSid(first: Sid, second: Sid) -> i32;
    fn FreeSid(sid: Sid) -> *mut c_void;
}

struct HandleGuard(Handle);

impl Drop for HandleGuard {
    fn drop(&mut self) {
        if !self.0.is_null() {
            // SAFETY: the handle was returned by CreateJobObjectW and is owned here.
            unsafe {
                CloseHandle(self.0);
            }
        }
    }
}

struct SidGuard(Sid);

impl Drop for SidGuard {
    fn drop(&mut self) {
        if !self.0.is_null() {
            // SAFETY: these SIDs are allocated by the documented Userenv APIs.
            unsafe {
                FreeSid(self.0);
            }
        }
    }
}

struct ProfileGuard {
    name: Vec<u16>,
    active: bool,
}

impl ProfileGuard {
    fn delete(&mut self) -> Result<(), String> {
        // SAFETY: name is a live, null-terminated UTF-16 string.
        let first = unsafe { DeleteAppContainerProfile(self.name.as_ptr()) };
        if hresult_succeeded(first) {
            self.active = false;
            return Ok(());
        }
        // Microsoft documents retrying deletion when the first result fails.
        // SAFETY: the same owned profile name remains valid for the retry.
        let second = unsafe { DeleteAppContainerProfile(self.name.as_ptr()) };
        if hresult_succeeded(second) {
            self.active = false;
            Ok(())
        } else {
            Err(format!(
                "AppContainer profile deletion failed: hresult={first:#010x}, retry={second:#010x}"
            ))
        }
    }
}

impl Drop for ProfileGuard {
    fn drop(&mut self) {
        if self.active {
            // SAFETY: name is a live, null-terminated UTF-16 string. The fresh
            // hosted VM is destroyed even if both best-effort calls fail.
            unsafe {
                DeleteAppContainerProfile(self.name.as_ptr());
                DeleteAppContainerProfile(self.name.as_ptr());
            }
        }
    }
}

fn wide(value: &str) -> Vec<u16> {
    OsStr::new(value).encode_wide().chain(Some(0)).collect()
}

fn hresult_succeeded(result: i32) -> bool {
    result >= 0
}

fn require_hosted_context() -> Result<(), String> {
    if env::var("GITHUB_ACTIONS").as_deref() != Ok("true")
        || env::var("RUNNER_ENVIRONMENT").as_deref() != Ok("github-hosted")
        || env::var("EXPECTED_WINDOWS_RUNNER").as_deref() != Ok("windows-2025")
    {
        return Err(
            "native probe is restricted to the fresh GitHub-hosted windows-2025 job".into(),
        );
    }
    if env::consts::OS != "windows" || env::consts::ARCH != "x86_64" {
        return Err("unsupported Windows target".into());
    }
    Ok(())
}

fn windows_build() -> Result<u32, String> {
    // SAFETY: zeroed is valid for this plain C structure and its size is set
    // before the documented RtlGetVersion call.
    let mut version: OsVersionInfoW = unsafe { zeroed() };
    version.dwOSVersionInfoSize = size_of::<OsVersionInfoW>() as u32;
    // SAFETY: version points to writable storage of the exact declared size.
    let status = unsafe { RtlGetVersion(&mut version) };
    if status != 0 || version.dwBuildNumber == 0 {
        return Err(format!("RtlGetVersion failed: status={status:#010x}"));
    }
    Ok(version.dwBuildNumber)
}

fn current_volume_is_ntfs() -> Result<(), String> {
    let current =
        env::current_dir().map_err(|error| format!("current directory unavailable: {error}"))?;
    let root = volume_root(&current)?;
    let mut filesystem = [0_u16; 32];
    // SAFETY: root is null-terminated and filesystem is a valid writable buffer;
    // every omitted output is documented optional.
    let ok = unsafe {
        GetVolumeInformationW(
            root.as_ptr(),
            null_mut(),
            0,
            null_mut(),
            null_mut(),
            null_mut(),
            filesystem.as_mut_ptr(),
            filesystem.len() as u32,
        )
    };
    if ok == 0 {
        return Err("GetVolumeInformationW failed".into());
    }
    let length = filesystem
        .iter()
        .position(|value| *value == 0)
        .unwrap_or(filesystem.len());
    let name = String::from_utf16(&filesystem[..length])
        .map_err(|_| "filesystem name was not valid UTF-16".to_owned())?;
    if !name.eq_ignore_ascii_case("NTFS") {
        return Err(format!("unsupported filesystem: {name}"));
    }
    Ok(())
}

fn volume_root(path: &Path) -> Result<Vec<u16>, String> {
    let encoded: Vec<u16> = path.as_os_str().encode_wide().collect();
    if encoded.len() < 3
        || !char::from_u32(u32::from(encoded[0])).is_some_and(|value| value.is_ascii_alphabetic())
        || encoded[1] != u16::from(b':')
        || (encoded[2] != u16::from(b'\\') && encoded[2] != u16::from(b'/'))
    {
        return Err("current directory is not on a drive-letter volume".into());
    }
    Ok(vec![encoded[0], u16::from(b':'), u16::from(b'\\'), 0])
}

fn require_kernel_apis() -> Result<(), String> {
    let kernel = wide("kernel32.dll");
    // SAFETY: kernel is a null-terminated module name and kernel32 is loaded in
    // every process using the linked functions above.
    let module = unsafe { GetModuleHandleW(kernel.as_ptr()) };
    if module.is_null() {
        return Err("kernel32 module handle unavailable".into());
    }
    for name in [
        b"InitializeProcThreadAttributeList\0".as_slice(),
        b"UpdateProcThreadAttribute\0".as_slice(),
        b"DeleteProcThreadAttributeList\0".as_slice(),
        b"CreateProcessW\0".as_slice(),
        b"GetProcessMitigationPolicy\0".as_slice(),
    ] {
        // SAFETY: each byte string is null-terminated and module is live.
        if unsafe { GetProcAddress(module, name.as_ptr()) }.is_null() {
            return Err("required process-launch or mitigation API unavailable".into());
        }
    }
    Ok(())
}

fn verify_empty_job_object() -> Result<(), String> {
    // SAFETY: null attributes and name request an unnamed job with default ACLs.
    let raw = unsafe { CreateJobObjectW(null(), null()) };
    if raw.is_null() {
        return Err("CreateJobObjectW failed".into());
    }
    let job = HandleGuard(raw);
    let mut requested = JobObjectExtendedLimitInformation::default();
    requested.BasicLimitInformation.LimitFlags =
        JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE | JOB_OBJECT_LIMIT_ACTIVE_PROCESS;
    requested.BasicLimitInformation.ActiveProcessLimit = 1;
    // SAFETY: job is live and requested has the exact information-class layout.
    let set = unsafe {
        SetInformationJobObject(
            job.0,
            JOB_OBJECT_EXTENDED_LIMIT_INFORMATION_CLASS,
            (&raw const requested).cast(),
            size_of::<JobObjectExtendedLimitInformation>() as u32,
        )
    };
    if set == 0 {
        return Err("SetInformationJobObject failed".into());
    }
    let mut observed = JobObjectExtendedLimitInformation::default();
    let mut returned_length = 0_u32;
    // SAFETY: observed is writable storage with the exact information-class layout.
    let queried = unsafe {
        QueryInformationJobObject(
            job.0,
            JOB_OBJECT_EXTENDED_LIMIT_INFORMATION_CLASS,
            (&raw mut observed).cast(),
            size_of::<JobObjectExtendedLimitInformation>() as u32,
            &raw mut returned_length,
        )
    };
    if queried == 0 || returned_length != size_of::<JobObjectExtendedLimitInformation>() as u32 {
        return Err("QueryInformationJobObject failed".into());
    }
    let flags = observed.BasicLimitInformation.LimitFlags;
    let required = JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE | JOB_OBJECT_LIMIT_ACTIVE_PROCESS;
    let forbidden = JOB_OBJECT_LIMIT_BREAKAWAY_OK | JOB_OBJECT_LIMIT_SILENT_BREAKAWAY_OK;
    if flags & required != required
        || flags & forbidden != 0
        || observed.BasicLimitInformation.ActiveProcessLimit != 1
        || flags & (JOB_OBJECT_LIMIT_PROCESS_MEMORY | JOB_OBJECT_LIMIT_JOB_MEMORY) != 0
    {
        return Err("Job Object query did not return the exact preflight limits".into());
    }
    Ok(())
}

fn verify_appcontainer_profile_lifecycle() -> Result<(), String> {
    // SAFETY: these fixed process-local identities do not expose user data.
    let process = unsafe { GetCurrentProcessId() };
    // SAFETY: monotonic uptime is used only to avoid a profile-name collision.
    let tick = unsafe { GetTickCount64() };
    let name = format!("studio.boldthaus.impresari.iar.{process}.{tick}");
    if name.len() > 64
        || !name.bytes().all(|value| {
            value.is_ascii_alphanumeric() || matches!(value, b'-' | b'_' | b'.' | b' ')
        })
    {
        return Err("generated AppContainer profile name violates the frozen contract".into());
    }
    let name = wide(&name);
    let display = wide("Impresari Context synthetic feasibility");
    let description = wide("Ephemeral zero-capability ADR-0092 profile");
    let mut created: Sid = null_mut();
    // SAFETY: all strings are live and null-terminated; capabilities is null
    // because the exact capability count is zero; created is writable.
    let result = unsafe {
        CreateAppContainerProfile(
            name.as_ptr(),
            display.as_ptr(),
            description.as_ptr(),
            null(),
            0,
            &raw mut created,
        )
    };
    if !hresult_succeeded(result) || created.is_null() {
        return Err(format!(
            "CreateAppContainerProfile failed: hresult={result:#010x}"
        ));
    }
    let mut profile = ProfileGuard { name, active: true };
    let created = SidGuard(created);
    let mut derived: Sid = null_mut();
    // SAFETY: profile.name remains live and derived is writable.
    let derived_result = unsafe {
        DeriveAppContainerSidFromAppContainerName(profile.name.as_ptr(), &raw mut derived)
    };
    if !hresult_succeeded(derived_result) || derived.is_null() {
        return Err(format!(
            "DeriveAppContainerSidFromAppContainerName failed: hresult={derived_result:#010x}"
        ));
    }
    let derived = SidGuard(derived);
    // SAFETY: both guards own valid SIDs returned by Userenv.
    if unsafe { EqualSid(created.0, derived.0) } == 0 {
        return Err("created and derived AppContainer SIDs differ".into());
    }
    profile.delete()
}

fn run() -> Result<u32, String> {
    require_hosted_context()?;
    let build = windows_build()?;
    current_volume_is_ntfs()?;
    require_kernel_apis()?;
    verify_empty_job_object()?;
    verify_appcontainer_profile_lifecycle()?;
    Ok(build)
}

fn main() {
    match run() {
        Ok(build) => println!(
            concat!(
                "{{\"schema_name\":\"windows-native-capability-preflight\",",
                "\"schema_version\":\"1.0.0\",",
                "\"profile_id\":\"{}\",\"profile_digest\":\"{}\",",
                "\"runner_environment\":\"github-hosted\",\"runner_label\":\"windows-2025\",",
                "\"os_family\":\"windows\",\"windows_build\":\"{}\",",
                "\"architecture\":\"x86_64\",\"filesystem\":\"ntfs\",",
                "\"required_launch_apis_present\":true,\"required_mitigation_apis_present\":true,",
                "\"job_object_created\":true,\"job_limits_set\":true,\"job_limits_queried\":true,",
                "\"job_kill_on_close_configurable\":true,\"active_process_limit_configurable\":true,",
                "\"breakaway_disabled\":true,\"appcontainer_profile_created\":true,",
                "\"appcontainer_sid_derived\":true,\"appcontainer_sid_matched\":true,",
                "\"appcontainer_profile_deleted\":true,\"capability_count\":\"0\",",
                "\"synthetic_worker_launched\":false,\"appcontainer_worker_launched\":false,",
                "\"network_denial_verified\":false,\"path_boundary_verified\":false,",
                "\"resource_limits_verified\":false,\"descendant_containment_verified\":false,",
                "\"complete_cleanup_verified\":false,\"os_confined\":false,",
                "\"production_admitted\":false,\"analyzer_execution\":false,",
                "\"authority_added\":false}}"
            ),
            PROFILE_ID, PROFILE_DIGEST, build
        ),
        Err(error) => {
            eprintln!("Windows native capability preflight failed: {error}");
            std::process::exit(1);
        }
    }
}
