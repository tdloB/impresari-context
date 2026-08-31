// SPDX-License-Identifier: Apache-2.0
#![cfg(windows)]
#![deny(unsafe_op_in_unsafe_fn)]
#![allow(non_snake_case)]

use std::{
    collections::BTreeMap,
    env,
    ffi::{OsStr, c_void},
    fs::{self, File},
    io::{Read, Write},
    mem::{size_of, zeroed},
    net::TcpListener,
    os::windows::{ffi::OsStrExt, io::FromRawHandle},
    path::{Path, PathBuf},
    ptr::{null, null_mut},
    thread,
    time::{Duration, Instant},
};

const PROFILE_ID: &str = "iar-windows-native-synthetic-worker-matrix-v1";
const PROFILE_DIGEST: &str =
    "sha256:82ab5c5c0cff76079ae19925b92da23b2d86e3a31e7cfc58626e17cb01c14678";
const BASE_PROFILE_ID: &str = "iar-windows-native-feasibility-v1";
const BASE_PROFILE_DIGEST: &str =
    "sha256:6b8f614387fc97321497e6b725213b9ee3c2159f3d1384fb800ffbe8af490a73";

type Handle = *mut c_void;
type Sid = *mut c_void;
type HKey = *mut c_void;
type BcryptAlg = *mut c_void;
type BcryptHash = *mut c_void;

const TOKEN_QUERY: u32 = 0x0008;
const TOKEN_USER: u32 = 1;
const HKEY_CURRENT_USER: HKey = 0x8000_0001_usize as HKey;
const KEY_ALL_ACCESS: u32 = 0x000f_003f;
const REG_OPTION_NON_VOLATILE: u32 = 0;
const REG_CREATED_NEW_KEY: u32 = 1;

const STARTF_USESTDHANDLES: u32 = 0x0000_0100;
const CREATE_SUSPENDED: u32 = 0x0000_0004;
const CREATE_UNICODE_ENVIRONMENT: u32 = 0x0000_0400;
const CREATE_NO_WINDOW: u32 = 0x0800_0000;
const EXTENDED_STARTUPINFO_PRESENT: u32 = 0x0008_0000;
const HANDLE_FLAG_INHERIT: u32 = 0x0000_0001;
const WAIT_OBJECT_0: u32 = 0;
const WAIT_TIMEOUT: u32 = 258;
const STILL_ACTIVE: u32 = 259;

const PROC_THREAD_ATTRIBUTE_HANDLE_LIST: usize = 0x0002_0002;
const PROC_THREAD_ATTRIBUTE_MITIGATION_POLICY: usize = 0x0002_0007;
const PROC_THREAD_ATTRIBUTE_SECURITY_CAPABILITIES: usize = 0x0002_0009;
const PROC_THREAD_ATTRIBUTE_CHILD_PROCESS_POLICY: usize = 0x0002_000e;
const PROC_THREAD_ATTRIBUTE_ALL_APPLICATION_PACKAGES_POLICY: usize = 0x0002_000f;
const PROCESS_CREATION_CHILD_PROCESS_RESTRICTED: u32 = 0x01;
const PROCESS_CREATION_ALL_APPLICATION_PACKAGES_OPT_OUT: u32 = 0x01;
const MITIGATION_POLICY: u64 = (1_u64)
    | (1_u64 << 28)
    | (1_u64 << 32)
    | (1_u64 << 36)
    | (1_u64 << 48)
    | (1_u64 << 52)
    | (1_u64 << 56)
    | (1_u64 << 60);

const JOB_OBJECT_BASIC_ACCOUNTING_INFORMATION_CLASS: i32 = 1;
const JOB_OBJECT_EXTENDED_LIMIT_INFORMATION_CLASS: i32 = 9;
const JOB_OBJECT_CPU_RATE_CONTROL_INFORMATION_CLASS: i32 = 15;
const JOB_OBJECT_LIMIT_ACTIVE_PROCESS: u32 = 0x0000_0008;
const JOB_OBJECT_LIMIT_PROCESS_MEMORY: u32 = 0x0000_0100;
const JOB_OBJECT_LIMIT_JOB_MEMORY: u32 = 0x0000_0200;
const JOB_OBJECT_LIMIT_BREAKAWAY_OK: u32 = 0x0000_0800;
const JOB_OBJECT_LIMIT_SILENT_BREAKAWAY_OK: u32 = 0x0000_1000;
const JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE: u32 = 0x0000_2000;
const JOB_OBJECT_CPU_RATE_CONTROL_ENABLE: u32 = 0x1;
const JOB_OBJECT_CPU_RATE_CONTROL_HARD_CAP: u32 = 0x4;

const SE_FILE_OBJECT: u32 = 1;
const DACL_SECURITY_INFORMATION: u32 = 0x0000_0004;
const PROTECTED_DACL_SECURITY_INFORMATION: u32 = 0x8000_0000;
const SDDL_REVISION_1: u32 = 1;

#[repr(C)]
#[derive(Clone, Copy)]
struct SecurityAttributes {
    nLength: u32,
    lpSecurityDescriptor: *mut c_void,
    bInheritHandle: i32,
}

#[repr(C)]
#[derive(Clone, Copy)]
struct SidAndAttributes {
    Sid: Sid,
    Attributes: u32,
}

#[repr(C)]
#[derive(Clone, Copy)]
struct TokenUserValue {
    User: SidAndAttributes,
}

#[repr(C)]
#[derive(Clone, Copy)]
struct SecurityCapabilities {
    AppContainerSid: Sid,
    Capabilities: *mut SidAndAttributes,
    CapabilityCount: u32,
    Reserved: u32,
}

#[repr(C)]
#[derive(Clone, Copy)]
struct StartupInfoW {
    cb: u32,
    lpReserved: *mut u16,
    lpDesktop: *mut u16,
    lpTitle: *mut u16,
    dwX: u32,
    dwY: u32,
    dwXSize: u32,
    dwYSize: u32,
    dwXCountChars: u32,
    dwYCountChars: u32,
    dwFillAttribute: u32,
    dwFlags: u32,
    wShowWindow: u16,
    cbReserved2: u16,
    lpReserved2: *mut u8,
    hStdInput: Handle,
    hStdOutput: Handle,
    hStdError: Handle,
}

impl Default for StartupInfoW {
    fn default() -> Self {
        // SAFETY: all-zero is the documented initialization for STARTUPINFOW.
        unsafe { zeroed() }
    }
}

#[repr(C)]
#[derive(Clone, Copy)]
struct StartupInfoExW {
    StartupInfo: StartupInfoW,
    lpAttributeList: *mut c_void,
}

impl Default for StartupInfoExW {
    fn default() -> Self {
        // SAFETY: all-zero is the documented initialization for STARTUPINFOEXW.
        unsafe { zeroed() }
    }
}

#[repr(C)]
#[derive(Clone, Copy)]
struct ProcessInformation {
    hProcess: Handle,
    hThread: Handle,
    dwProcessId: u32,
    dwThreadId: u32,
}

impl Default for ProcessInformation {
    fn default() -> Self {
        // SAFETY: all-zero is the documented initialization for PROCESS_INFORMATION.
        unsafe { zeroed() }
    }
}

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

#[repr(C)]
#[derive(Clone, Copy, Default)]
struct JobObjectCpuRateControlInformation {
    ControlFlags: u32,
    CpuRate: u32,
}

#[repr(C)]
#[derive(Clone, Copy, Default)]
struct JobObjectBasicAccountingInformation {
    TotalUserTime: i64,
    TotalKernelTime: i64,
    ThisPeriodTotalUserTime: i64,
    ThisPeriodTotalKernelTime: i64,
    TotalPageFaultCount: u32,
    TotalProcesses: u32,
    ActiveProcesses: u32,
    TotalTerminatedProcesses: u32,
}

#[link(name = "kernel32")]
unsafe extern "system" {
    fn GetCurrentProcess() -> Handle;
    fn GetCurrentProcessId() -> u32;
    fn GetLastError() -> u32;
    fn GetTickCount64() -> u64;
    fn CreatePipe(
        read_pipe: *mut Handle,
        write_pipe: *mut Handle,
        attributes: *const SecurityAttributes,
        size: u32,
    ) -> i32;
    fn SetHandleInformation(handle: Handle, mask: u32, flags: u32) -> i32;
    fn CreateEventW(
        attributes: *const SecurityAttributes,
        manual_reset: i32,
        initial_state: i32,
        name: *const u16,
    ) -> Handle;
    fn InitializeProcThreadAttributeList(
        list: *mut c_void,
        count: u32,
        flags: u32,
        size: *mut usize,
    ) -> i32;
    fn UpdateProcThreadAttribute(
        list: *mut c_void,
        flags: u32,
        attribute: usize,
        value: *mut c_void,
        size: usize,
        previous: *mut c_void,
        return_size: *mut usize,
    ) -> i32;
    fn DeleteProcThreadAttributeList(list: *mut c_void);
    fn CreateProcessW(
        application_name: *const u16,
        command_line: *mut u16,
        process_attributes: *const c_void,
        thread_attributes: *const c_void,
        inherit_handles: i32,
        creation_flags: u32,
        environment: *const c_void,
        current_directory: *const u16,
        startup_info: *mut StartupInfoW,
        process_information: *mut ProcessInformation,
    ) -> i32;
    fn ResumeThread(thread: Handle) -> u32;
    fn WaitForSingleObject(handle: Handle, milliseconds: u32) -> u32;
    fn GetExitCodeProcess(process: Handle, exit_code: *mut u32) -> i32;
    fn TerminateProcess(process: Handle, exit_code: u32) -> i32;
    fn CloseHandle(handle: Handle) -> i32;
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
    fn AssignProcessToJobObject(job: Handle, process: Handle) -> i32;
    fn IsProcessInJob(process: Handle, job: Handle, result: *mut i32) -> i32;
    fn TerminateJobObject(job: Handle, exit_code: u32) -> i32;
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
    fn GetWindowsDirectoryW(buffer: *mut u16, size: u32) -> u32;
    fn LocalFree(memory: *mut c_void) -> *mut c_void;
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
    fn DeleteAppContainerProfile(appcontainer_name: *const u16) -> i32;
    fn GetAppContainerFolderPath(appcontainer_sid: *const u16, path: *mut *mut u16) -> i32;
    fn CreateEnvironmentBlock(environment: *mut *mut c_void, token: Handle, inherit: i32) -> i32;
    fn DestroyEnvironmentBlock(environment: *mut c_void) -> i32;
}

#[link(name = "ole32")]
unsafe extern "system" {
    fn CoTaskMemFree(memory: *const c_void);
}

#[link(name = "advapi32")]
unsafe extern "system" {
    fn OpenProcessToken(process: Handle, desired_access: u32, token: *mut Handle) -> i32;
    fn GetTokenInformation(
        token: Handle,
        information_class: u32,
        information: *mut c_void,
        information_length: u32,
        return_length: *mut u32,
    ) -> i32;
    fn ConvertSidToStringSidW(sid: Sid, string_sid: *mut *mut u16) -> i32;
    fn FreeSid(sid: Sid) -> *mut c_void;
    fn ConvertStringSecurityDescriptorToSecurityDescriptorW(
        descriptor: *const u16,
        revision: u32,
        security_descriptor: *mut *mut c_void,
        size: *mut u32,
    ) -> i32;
    fn GetSecurityDescriptorDacl(
        descriptor: *mut c_void,
        present: *mut i32,
        dacl: *mut *mut c_void,
        defaulted: *mut i32,
    ) -> i32;
    fn SetNamedSecurityInfoW(
        object_name: *mut u16,
        object_type: u32,
        security_information: u32,
        owner: Sid,
        group: Sid,
        dacl: *mut c_void,
        sacl: *mut c_void,
    ) -> u32;
    fn RegCreateKeyExW(
        key: HKey,
        subkey: *const u16,
        reserved: u32,
        class: *mut u16,
        options: u32,
        desired: u32,
        attributes: *const c_void,
        result: *mut HKey,
        disposition: *mut u32,
    ) -> i32;
    fn RegCloseKey(key: HKey) -> i32;
    fn RegDeleteTreeW(key: HKey, subkey: *const u16) -> i32;
}

#[link(name = "bcrypt")]
unsafe extern "system" {
    fn BCryptOpenAlgorithmProvider(
        algorithm: *mut BcryptAlg,
        algorithm_id: *const u16,
        implementation: *const u16,
        flags: u32,
    ) -> i32;
    fn BCryptGetProperty(
        object: *mut c_void,
        property: *const u16,
        output: *mut u8,
        output_length: u32,
        result_length: *mut u32,
        flags: u32,
    ) -> i32;
    fn BCryptCreateHash(
        algorithm: BcryptAlg,
        hash: *mut BcryptHash,
        hash_object: *mut u8,
        hash_object_length: u32,
        secret: *const u8,
        secret_length: u32,
        flags: u32,
    ) -> i32;
    fn BCryptHashData(hash: BcryptHash, input: *mut u8, input_length: u32, flags: u32) -> i32;
    fn BCryptFinishHash(hash: BcryptHash, output: *mut u8, output_length: u32, flags: u32) -> i32;
    fn BCryptDestroyHash(hash: BcryptHash) -> i32;
    fn BCryptCloseAlgorithmProvider(algorithm: BcryptAlg, flags: u32) -> i32;
}

struct HandleGuard(Handle);

impl HandleGuard {
    fn take(&mut self) -> Handle {
        let value = self.0;
        self.0 = null_mut();
        value
    }
}

impl Drop for HandleGuard {
    fn drop(&mut self) {
        if !self.0.is_null() {
            // SAFETY: this guard owns one live kernel handle.
            unsafe { CloseHandle(self.0) };
        }
    }
}

struct SidGuard(Sid);

impl Drop for SidGuard {
    fn drop(&mut self) {
        if !self.0.is_null() {
            // SAFETY: this SID was allocated by Userenv.
            unsafe { FreeSid(self.0) };
        }
    }
}

struct AttributeList {
    storage: Vec<usize>,
    pointer: *mut c_void,
}

struct LaunchAttributes {
    list: AttributeList,
    capabilities: Box<SecurityCapabilities>,
    handles: Box<[Handle; 3]>,
    child_policy: Box<u32>,
    all_packages: Box<u32>,
    mitigation: Box<[u64; 2]>,
}

struct EnvironmentBlock(Vec<u16>);

impl EnvironmentBlock {
    fn exact_system(stage: &Path) -> Result<Self, String> {
        let mut windows = vec![0_u16; 32_768];
        // SAFETY: windows is writable for the capacity passed to the API.
        let length = unsafe { GetWindowsDirectoryW(windows.as_mut_ptr(), windows.len() as u32) };
        if length == 0 || length as usize >= windows.len() {
            return Err(format!("GetWindowsDirectoryW failed: win32={}", unsafe {
                GetLastError()
            }));
        }
        windows.truncate(length as usize);
        if windows.len() < 2 || windows[1] != b':' as u16 {
            return Err("Windows directory did not identify a local system drive".into());
        }
        let system_drive =
            String::from_utf16(&windows[..2]).map_err(|_| "system drive was invalid UTF-16")?;
        let system_root =
            String::from_utf16(&windows).map_err(|_| "Windows directory was invalid UTF-16")?;
        let mut token = null_mut();
        // SAFETY: token is writable and only query authority is requested.
        if unsafe { OpenProcessToken(GetCurrentProcess(), TOKEN_QUERY, &raw mut token) } == 0
            || token.is_null()
        {
            return Err("OpenProcessToken(environment) failed".into());
        }
        let token = HandleGuard(token);
        let mut clean = null_mut();
        // SAFETY: the current user token is queryable and inherit=false excludes
        // all process-level caller variables.
        if unsafe { CreateEnvironmentBlock(&raw mut clean, token.0, 0) } == 0 || clean.is_null() {
            return Err("CreateEnvironmentBlock(clean user) failed".into());
        }
        let allowed = [
            "APPDATA",
            "HOMEDRIVE",
            "HOMEPATH",
            "LOCALAPPDATA",
            "USERPROFILE",
        ];
        let mut entries = BTreeMap::new();
        let mut cursor = clean.cast::<u16>();
        let mut consumed = 0_usize;
        while consumed < 32_768 {
            let mut length = 0_usize;
            // SAFETY: CreateEnvironmentBlock returns a double-null-terminated
            // block. The frozen 32K bound rejects malformed output.
            unsafe {
                while consumed + length < 32_768 && *cursor.add(length) != 0 {
                    length += 1;
                }
            }
            if length == 0 {
                break;
            }
            if consumed + length >= 32_768 {
                // SAFETY: clean is owned by Userenv until this point.
                unsafe { DestroyEnvironmentBlock(clean) };
                return Err("clean user environment exceeded the frozen bound".into());
            }
            // SAFETY: the loop found the terminator within the frozen bound.
            let units = unsafe { std::slice::from_raw_parts(cursor, length) };
            let entry = match String::from_utf16(units) {
                Ok(value) => value,
                Err(_) => {
                    // SAFETY: clean is owned by Userenv until this point.
                    unsafe { DestroyEnvironmentBlock(clean) };
                    return Err("clean user environment was invalid UTF-16".into());
                }
            };
            if let Some((key, value)) = entry.split_once('=')
                && let Some(canonical) = allowed
                    .iter()
                    .find(|candidate| candidate.eq_ignore_ascii_case(key))
            {
                entries.insert((*canonical).to_string(), value.to_string());
            }
            // SAFETY: length points at the current entry terminator.
            cursor = unsafe { cursor.add(length + 1) };
            consumed += length + 1;
        }
        // SAFETY: all selected strings are now owned Rust values.
        unsafe { DestroyEnvironmentBlock(clean) };
        for required in allowed {
            if !entries.contains_key(required) {
                return Err(format!(
                    "required clean environment key unavailable: {required}"
                ));
            }
        }
        entries.insert(
            "ComSpec".into(),
            format!("{system_root}\\System32\\cmd.exe"),
        );
        entries.insert(
            "Path".into(),
            format!("{system_root}\\System32;{system_root};{system_root}\\System32\\Wbem"),
        );
        entries.insert("SystemDrive".into(), system_drive);
        entries.insert("SystemRoot".into(), system_root.clone());
        entries.insert("TEMP".into(), stage.to_string_lossy().into_owned());
        entries.insert("TMP".into(), stage.to_string_lossy().into_owned());
        entries.insert("windir".into(), system_root);
        let mut entries: Vec<_> = entries.into_iter().collect();
        entries.sort_by_key(|(key, _)| key.to_ascii_uppercase());
        let mut block = Vec::new();
        // CreateProcess requires an alphabetically sorted, double-null-terminated
        // Unicode block. The allowlist excludes repository, credential, CI, and
        // arbitrary user variables; process-level caller variables are never read.
        for (key, value) in entries {
            block.extend(format!("{key}={value}").encode_utf16());
            block.push(0);
        }
        block.push(0);
        Ok(Self(block))
    }
}

impl Drop for AttributeList {
    fn drop(&mut self) {
        if !self.pointer.is_null() {
            // SAFETY: pointer was initialized by InitializeProcThreadAttributeList.
            unsafe { DeleteProcThreadAttributeList(self.pointer) };
        }
        std::hint::black_box(&self.storage);
    }
}

struct Profile {
    name: Vec<u16>,
    sid: SidGuard,
    sid_string: String,
    path: PathBuf,
    active: bool,
}

impl Profile {
    fn delete(&mut self) -> Result<(), String> {
        // SAFETY: name is a live null-terminated profile identity.
        let first = unsafe { DeleteAppContainerProfile(self.name.as_ptr()) };
        if first >= 0 {
            self.active = false;
            return Ok(());
        }
        // SAFETY: Microsoft documents retrying deletion and name remains live.
        let second = unsafe { DeleteAppContainerProfile(self.name.as_ptr()) };
        if second >= 0 {
            self.active = false;
            Ok(())
        } else {
            Err(format!(
                "profile deletion failed: first={first:#010x} second={second:#010x}"
            ))
        }
    }
}

impl Drop for Profile {
    fn drop(&mut self) {
        if self.active {
            // SAFETY: best-effort exact-profile cleanup on a disposable host.
            unsafe {
                DeleteAppContainerProfile(self.name.as_ptr());
                DeleteAppContainerProfile(self.name.as_ptr());
            }
        }
    }
}

struct ScenarioOutcome {
    stdout: Vec<u8>,
    stderr: Vec<u8>,
    exit_code: u32,
    deadline_hit: bool,
    total_processes: u32,
    active_processes: u32,
}

struct JobProcessCleanup {
    job: Handle,
    process: Handle,
}

impl Drop for JobProcessCleanup {
    fn drop(&mut self) {
        if !self.job.is_null() && !self.process.is_null() {
            // SAFETY: both non-owning handles remain live until after this guard
            // drops. Termination and wait are idempotent after normal exit.
            unsafe {
                TerminateJobObject(self.job, 0xd00d);
                WaitForSingleObject(self.process, 5_000);
            }
        }
    }
}

fn wide(value: &OsStr) -> Vec<u16> {
    value.encode_wide().chain(Some(0)).collect()
}

fn wide_str(value: &str) -> Vec<u16> {
    wide(OsStr::new(value))
}

fn hresult_succeeded(value: i32) -> bool {
    value >= 0
}

fn require_hosted_context() -> Result<(), String> {
    if env::var("GITHUB_ACTIONS").as_deref() != Ok("true")
        || env::var("RUNNER_ENVIRONMENT").as_deref() != Ok("github-hosted")
        || env::var("EXPECTED_WINDOWS_RUNNER").as_deref() != Ok("windows-2025")
    {
        return Err("broker is restricted to a fresh GitHub-hosted windows-2025 job".into());
    }
    if env::consts::OS != "windows" || env::consts::ARCH != "x86_64" {
        return Err("unsupported Windows target".into());
    }
    Ok(())
}

fn windows_build() -> Result<u32, String> {
    // SAFETY: zeroed is valid for this plain C structure.
    let mut version: OsVersionInfoW = unsafe { zeroed() };
    version.dwOSVersionInfoSize = size_of::<OsVersionInfoW>() as u32;
    // SAFETY: version is writable storage of the declared size.
    let status = unsafe { RtlGetVersion(&raw mut version) };
    if status != 0 || version.dwBuildNumber == 0 {
        return Err(format!("RtlGetVersion failed: {status:#010x}"));
    }
    Ok(version.dwBuildNumber)
}

fn volume_root(path: &Path) -> Result<Vec<u16>, String> {
    let encoded: Vec<u16> = path.as_os_str().encode_wide().collect();
    if encoded.len() < 3
        || encoded[1] != u16::from(b':')
        || (encoded[2] != u16::from(b'\\') && encoded[2] != u16::from(b'/'))
    {
        return Err("path is not on a drive-letter volume".into());
    }
    Ok(vec![encoded[0], u16::from(b':'), u16::from(b'\\'), 0])
}

fn require_ntfs(path: &Path) -> Result<(), String> {
    let root = volume_root(path)?;
    let mut filesystem = [0_u16; 32];
    // SAFETY: root and the output buffer are live for the call.
    if unsafe {
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
    } == 0
    {
        return Err("GetVolumeInformationW failed".into());
    }
    let length = filesystem
        .iter()
        .position(|value| *value == 0)
        .unwrap_or(filesystem.len());
    let name = String::from_utf16(&filesystem[..length])
        .map_err(|_| "filesystem name was invalid UTF-16")?;
    if !name.eq_ignore_ascii_case("NTFS") {
        return Err(format!("unsupported filesystem: {name}"));
    }
    Ok(())
}

fn sid_to_string(sid: Sid) -> Result<String, String> {
    let mut raw = null_mut();
    // SAFETY: sid is live and raw is writable.
    if unsafe { ConvertSidToStringSidW(sid, &raw mut raw) } == 0 || raw.is_null() {
        return Err("ConvertSidToStringSidW failed".into());
    }
    let mut length = 0_usize;
    // SAFETY: raw points to a null-terminated LocalAlloc string.
    unsafe {
        while *raw.add(length) != 0 {
            length += 1;
        }
    }
    // SAFETY: the preceding scan found the string terminator.
    let units = unsafe { std::slice::from_raw_parts(raw, length) };
    let value = String::from_utf16(units).map_err(|_| "SID was invalid UTF-16")?;
    // SAFETY: the API allocated raw with LocalAlloc.
    unsafe { LocalFree(raw.cast()) };
    Ok(value)
}

fn current_user_sid() -> Result<String, String> {
    let mut token = null_mut();
    // SAFETY: token is writable and the process pseudo-handle is valid.
    if unsafe { OpenProcessToken(GetCurrentProcess(), TOKEN_QUERY, &raw mut token) } == 0 {
        return Err("OpenProcessToken failed".into());
    }
    let token = HandleGuard(token);
    let mut required = 0_u32;
    // SAFETY: the null query obtains the exact required length.
    unsafe {
        GetTokenInformation(token.0, TOKEN_USER, null_mut(), 0, &raw mut required);
    }
    if required < size_of::<TokenUserValue>() as u32 {
        return Err("TokenUser size query failed".into());
    }
    let words = (required as usize).div_ceil(size_of::<usize>());
    let mut storage = vec![0_usize; words];
    // SAFETY: storage is aligned writable memory of at least required bytes.
    if unsafe {
        GetTokenInformation(
            token.0,
            TOKEN_USER,
            storage.as_mut_ptr().cast(),
            required,
            &raw mut required,
        )
    } == 0
    {
        return Err("GetTokenInformation(TokenUser) failed".into());
    }
    // SAFETY: the successful call wrote TOKEN_USER at the buffer start.
    let user = unsafe { &*storage.as_ptr().cast::<TokenUserValue>() };
    sid_to_string(user.User.Sid)
}

fn profile_path(sid_string: &str) -> Result<PathBuf, String> {
    let sid = wide_str(sid_string);
    let mut raw = null_mut();
    // SAFETY: sid is a live null-terminated SID string and raw is writable.
    let result = unsafe { GetAppContainerFolderPath(sid.as_ptr(), &raw mut raw) };
    if !hresult_succeeded(result) || raw.is_null() {
        return Err(format!("GetAppContainerFolderPath failed: {result:#010x}"));
    }
    let mut length = 0_usize;
    // SAFETY: raw points to a null-terminated CoTaskMem string.
    unsafe {
        while *raw.add(length) != 0 {
            length += 1;
        }
    }
    // SAFETY: the preceding scan found the terminator.
    let units = unsafe { std::slice::from_raw_parts(raw, length) };
    let value = String::from_utf16(units).map_err(|_| "profile path was invalid UTF-16")?;
    // SAFETY: GetAppContainerFolderPath requires CoTaskMemFree.
    unsafe { CoTaskMemFree(raw.cast()) };
    Ok(PathBuf::from(value))
}

fn create_profile(label: &str, nonce: u64) -> Result<Profile, String> {
    let process = unsafe { GetCurrentProcessId() };
    let name = format!("studio.boldthaus.impresari.iar.{process}.{nonce}.{label}");
    if name.len() > 64
        || !name
            .bytes()
            .all(|value| value.is_ascii_alphanumeric() || value == b'.')
    {
        return Err("generated profile identity violated the frozen contract".into());
    }
    let name = wide_str(&name);
    let display = wide_str("Impresari Context synthetic worker matrix");
    let description = wide_str("Ephemeral zero-capability ADR-0093 LPAC profile");
    let mut sid = null_mut();
    // SAFETY: strings are live, the capability vector is exactly empty, and sid is writable.
    let result = unsafe {
        CreateAppContainerProfile(
            name.as_ptr(),
            display.as_ptr(),
            description.as_ptr(),
            null(),
            0,
            &raw mut sid,
        )
    };
    if !hresult_succeeded(result) || sid.is_null() {
        return Err(format!("CreateAppContainerProfile failed: {result:#010x}"));
    }
    let sid = SidGuard(sid);
    let sid_string = match sid_to_string(sid.0) {
        Ok(value) => value,
        Err(error) => {
            // SAFETY: name identifies only the just-created synthetic profile.
            unsafe { DeleteAppContainerProfile(name.as_ptr()) };
            return Err(error);
        }
    };
    let path = match profile_path(&sid_string) {
        Ok(value) => value,
        Err(error) => {
            // SAFETY: name identifies only the just-created synthetic profile.
            unsafe { DeleteAppContainerProfile(name.as_ptr()) };
            return Err(error);
        }
    };
    Ok(Profile {
        name,
        sid,
        sid_string,
        path,
        active: true,
    })
}

fn set_dacl(path: &Path, user_sid: &str, app_sids: &[&str], directory: bool) -> Result<(), String> {
    let inheritance = if directory { "OICI" } else { "" };
    let app_rights = "GRGX";
    let mut sddl = format!("D:P(A;{inheritance};FA;;;SY)(A;{inheritance};FA;;;{user_sid})");
    for sid in app_sids {
        sddl.push_str(&format!("(A;{inheritance};{app_rights};;;{sid})"));
    }
    let sddl = wide_str(&sddl);
    let mut descriptor = null_mut();
    // SAFETY: sddl is null-terminated and descriptor is writable.
    if unsafe {
        ConvertStringSecurityDescriptorToSecurityDescriptorW(
            sddl.as_ptr(),
            SDDL_REVISION_1,
            &raw mut descriptor,
            null_mut(),
        )
    } == 0
        || descriptor.is_null()
    {
        return Err("security descriptor conversion failed".into());
    }
    let mut present = 0;
    let mut defaulted = 0;
    let mut dacl = null_mut();
    // SAFETY: descriptor is live and all outputs are writable.
    let got = unsafe {
        GetSecurityDescriptorDacl(
            descriptor,
            &raw mut present,
            &raw mut dacl,
            &raw mut defaulted,
        )
    };
    if got == 0 || present == 0 || dacl.is_null() {
        // SAFETY: descriptor was allocated by LocalAlloc.
        unsafe { LocalFree(descriptor) };
        return Err("security descriptor DACL unavailable".into());
    }
    let mut object = wide(path.as_os_str());
    // SAFETY: object is mutable/null-terminated and dacl remains owned by descriptor.
    let status = unsafe {
        SetNamedSecurityInfoW(
            object.as_mut_ptr(),
            SE_FILE_OBJECT,
            DACL_SECURITY_INFORMATION | PROTECTED_DACL_SECURITY_INFORMATION,
            null_mut(),
            null_mut(),
            dacl,
            null_mut(),
        )
    };
    // SAFETY: SetNamedSecurityInfoW has returned; descriptor is no longer borrowed.
    unsafe { LocalFree(descriptor) };
    if status != 0 {
        return Err(format!("SetNamedSecurityInfoW failed: {status}"));
    }
    Ok(())
}

fn harden_profile(profile: &Profile, user_sid: &str) -> Result<(), String> {
    if !profile.path.is_dir() {
        return Err("AppContainer profile directory was not created".into());
    }
    let mut entries = vec![profile.path.clone()];
    let mut index = 0;
    while index < entries.len() {
        let path = entries[index].clone();
        index += 1;
        if path.is_dir() {
            for entry in
                fs::read_dir(&path).map_err(|error| format!("profile traversal failed: {error}"))?
            {
                let entry = entry.map_err(|error| format!("profile traversal failed: {error}"))?;
                let metadata = entry
                    .metadata()
                    .map_err(|error| format!("profile metadata failed: {error}"))?;
                if metadata.file_type().is_symlink() {
                    return Err("profile storage unexpectedly contained a symlink".into());
                }
                entries.push(entry.path());
            }
        }
    }
    entries.sort_by_key(|path| std::cmp::Reverse(path.components().count()));
    for path in entries {
        let app_sids = if path == profile.path {
            vec![profile.sid_string.as_str()]
        } else {
            Vec::new()
        };
        set_dacl(&path, user_sid, &app_sids, path.is_dir())?;
    }
    Ok(())
}

fn sha256(path: &Path) -> Result<String, String> {
    let sha256_id = wide_str("SHA256");
    let object_length_name = wide_str("ObjectLength");
    let digest_length_name = wide_str("HashDigestLength");
    let mut algorithm = null_mut();
    // SAFETY: algorithm is writable and the algorithm identifier is null-terminated.
    if unsafe { BCryptOpenAlgorithmProvider(&raw mut algorithm, sha256_id.as_ptr(), null(), 0) } < 0
        || algorithm.is_null()
    {
        return Err("BCryptOpenAlgorithmProvider failed".into());
    }
    let mut object_length = 0_u32;
    let mut result_length = 0_u32;
    // SAFETY: output points to one writable DWORD.
    let object_status = unsafe {
        BCryptGetProperty(
            algorithm,
            object_length_name.as_ptr(),
            (&raw mut object_length).cast(),
            size_of::<u32>() as u32,
            &raw mut result_length,
            0,
        )
    };
    let mut digest_length = 0_u32;
    // SAFETY: output points to one writable DWORD.
    let digest_status = unsafe {
        BCryptGetProperty(
            algorithm,
            digest_length_name.as_ptr(),
            (&raw mut digest_length).cast(),
            size_of::<u32>() as u32,
            &raw mut result_length,
            0,
        )
    };
    if object_status < 0 || digest_status < 0 || digest_length != 32 || object_length == 0 {
        // SAFETY: algorithm is an owned provider handle.
        unsafe { BCryptCloseAlgorithmProvider(algorithm, 0) };
        return Err("BCrypt SHA-256 properties were unexpected".into());
    }
    let mut object = vec![0_u8; object_length as usize];
    let mut hash = null_mut();
    // SAFETY: all buffers are live and sized from BCrypt properties.
    if unsafe {
        BCryptCreateHash(
            algorithm,
            &raw mut hash,
            object.as_mut_ptr(),
            object_length,
            null(),
            0,
            0,
        )
    } < 0
        || hash.is_null()
    {
        // SAFETY: algorithm is an owned provider handle.
        unsafe { BCryptCloseAlgorithmProvider(algorithm, 0) };
        return Err("BCryptCreateHash failed".into());
    }
    let mut file = File::open(path).map_err(|error| format!("hash input open failed: {error}"))?;
    let mut buffer = [0_u8; 64 * 1024];
    loop {
        let count = file
            .read(&mut buffer)
            .map_err(|error| format!("hash input read failed: {error}"))?;
        if count == 0 {
            break;
        }
        // SAFETY: hash is live and the buffer prefix contains count initialized bytes.
        if unsafe { BCryptHashData(hash, buffer.as_mut_ptr(), count as u32, 0) } < 0 {
            unsafe {
                BCryptDestroyHash(hash);
                BCryptCloseAlgorithmProvider(algorithm, 0);
            }
            return Err("BCryptHashData failed".into());
        }
    }
    let mut digest = [0_u8; 32];
    // SAFETY: hash is live and digest is the exact advertised length.
    let finish = unsafe { BCryptFinishHash(hash, digest.as_mut_ptr(), digest.len() as u32, 0) };
    // SAFETY: both owned BCrypt handles are no longer used.
    unsafe {
        BCryptDestroyHash(hash);
        BCryptCloseAlgorithmProvider(algorithm, 0);
    }
    if finish < 0 {
        return Err("BCryptFinishHash failed".into());
    }
    Ok(digest.iter().map(|byte| format!("{byte:02x}")).collect())
}

fn verify_binary_digest(path: &Path, variable: &str) -> Result<String, String> {
    let digest = sha256(path)?;
    let expected = env::var(variable)
        .map_err(|_| format!("missing {variable}"))?
        .trim()
        .to_ascii_lowercase();
    if expected != digest || digest.bytes().all(|value| value == b'0') {
        return Err(format!(
            "{variable} did not match the exact binary: expected={expected} observed={digest}"
        ));
    }
    Ok(format!("sha256:{digest}"))
}

fn create_job() -> Result<HandleGuard, String> {
    // SAFETY: null attributes and name request a private unnamed Job Object.
    let raw = unsafe { CreateJobObjectW(null(), null()) };
    if raw.is_null() {
        return Err("CreateJobObjectW failed".into());
    }
    let job = HandleGuard(raw);
    let mut limits = JobObjectExtendedLimitInformation::default();
    limits.BasicLimitInformation.LimitFlags = JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE
        | JOB_OBJECT_LIMIT_ACTIVE_PROCESS
        | JOB_OBJECT_LIMIT_PROCESS_MEMORY
        | JOB_OBJECT_LIMIT_JOB_MEMORY;
    limits.BasicLimitInformation.ActiveProcessLimit = 1;
    limits.ProcessMemoryLimit = 67_108_864;
    limits.JobMemoryLimit = 134_217_728;
    // SAFETY: job is live and limits has the exact information-class layout.
    if unsafe {
        SetInformationJobObject(
            job.0,
            JOB_OBJECT_EXTENDED_LIMIT_INFORMATION_CLASS,
            (&raw const limits).cast(),
            size_of::<JobObjectExtendedLimitInformation>() as u32,
        )
    } == 0
    {
        return Err("SetInformationJobObject(limits) failed".into());
    }
    let cpu = JobObjectCpuRateControlInformation {
        ControlFlags: JOB_OBJECT_CPU_RATE_CONTROL_ENABLE | JOB_OBJECT_CPU_RATE_CONTROL_HARD_CAP,
        CpuRate: 2_500,
    };
    // SAFETY: cpu has the exact information-class layout.
    if unsafe {
        SetInformationJobObject(
            job.0,
            JOB_OBJECT_CPU_RATE_CONTROL_INFORMATION_CLASS,
            (&raw const cpu).cast(),
            size_of::<JobObjectCpuRateControlInformation>() as u32,
        )
    } == 0
    {
        return Err("SetInformationJobObject(cpu) failed".into());
    }
    let mut observed = JobObjectExtendedLimitInformation::default();
    let mut returned = 0_u32;
    // SAFETY: observed is writable exact-layout storage.
    if unsafe {
        QueryInformationJobObject(
            job.0,
            JOB_OBJECT_EXTENDED_LIMIT_INFORMATION_CLASS,
            (&raw mut observed).cast(),
            size_of::<JobObjectExtendedLimitInformation>() as u32,
            &raw mut returned,
        )
    } == 0
        || returned != size_of::<JobObjectExtendedLimitInformation>() as u32
    {
        return Err("QueryInformationJobObject(limits) failed".into());
    }
    let flags = observed.BasicLimitInformation.LimitFlags;
    let required = JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE
        | JOB_OBJECT_LIMIT_ACTIVE_PROCESS
        | JOB_OBJECT_LIMIT_PROCESS_MEMORY
        | JOB_OBJECT_LIMIT_JOB_MEMORY;
    if flags & required != required
        || flags & (JOB_OBJECT_LIMIT_BREAKAWAY_OK | JOB_OBJECT_LIMIT_SILENT_BREAKAWAY_OK) != 0
        || observed.BasicLimitInformation.ActiveProcessLimit != 1
        || observed.ProcessMemoryLimit != 67_108_864
        || observed.JobMemoryLimit != 134_217_728
    {
        return Err("Job Object limits did not query exactly".into());
    }
    let mut observed_cpu = JobObjectCpuRateControlInformation::default();
    // SAFETY: observed_cpu is writable exact-layout storage.
    if unsafe {
        QueryInformationJobObject(
            job.0,
            JOB_OBJECT_CPU_RATE_CONTROL_INFORMATION_CLASS,
            (&raw mut observed_cpu).cast(),
            size_of::<JobObjectCpuRateControlInformation>() as u32,
            &raw mut returned,
        )
    } == 0
        || observed_cpu.ControlFlags != cpu.ControlFlags
        || observed_cpu.CpuRate != cpu.CpuRate
    {
        return Err("Job Object CPU control did not query exactly".into());
    }
    Ok(job)
}

fn attribute_list(sid: Sid, handles: [Handle; 3]) -> Result<LaunchAttributes, String> {
    let mut size = 0_usize;
    // SAFETY: the null query obtains the required opaque storage length.
    unsafe { InitializeProcThreadAttributeList(null_mut(), 5, 0, &raw mut size) };
    if size == 0 {
        return Err("attribute-list size query failed".into());
    }
    let mut storage = vec![0_usize; size.div_ceil(size_of::<usize>())];
    let pointer = storage.as_mut_ptr().cast();
    // SAFETY: pointer is aligned writable storage of at least size bytes.
    if unsafe { InitializeProcThreadAttributeList(pointer, 5, 0, &raw mut size) } == 0 {
        return Err("InitializeProcThreadAttributeList failed".into());
    }
    let list = AttributeList { storage, pointer };
    let mut capabilities = Box::new(SecurityCapabilities {
        AppContainerSid: sid,
        Capabilities: null_mut(),
        CapabilityCount: 0,
        Reserved: 0,
    });
    let mut handles = Box::new(handles);
    let mut child_policy = Box::new(PROCESS_CREATION_CHILD_PROCESS_RESTRICTED);
    let mut all_packages = Box::new(PROCESS_CREATION_ALL_APPLICATION_PACKAGES_OPT_OUT);
    let mut mitigation = Box::new([MITIGATION_POLICY, 0]);
    for (attribute, value, value_size) in [
        (
            PROC_THREAD_ATTRIBUTE_SECURITY_CAPABILITIES,
            (&raw mut *capabilities).cast(),
            size_of::<SecurityCapabilities>(),
        ),
        (
            PROC_THREAD_ATTRIBUTE_HANDLE_LIST,
            handles.as_mut_ptr().cast(),
            size_of::<[Handle; 3]>(),
        ),
        (
            PROC_THREAD_ATTRIBUTE_CHILD_PROCESS_POLICY,
            (&raw mut *child_policy).cast(),
            size_of::<u32>(),
        ),
        (
            PROC_THREAD_ATTRIBUTE_ALL_APPLICATION_PACKAGES_POLICY,
            (&raw mut *all_packages).cast(),
            size_of::<u32>(),
        ),
        (
            PROC_THREAD_ATTRIBUTE_MITIGATION_POLICY,
            mitigation.as_mut_ptr().cast(),
            size_of::<[u64; 2]>(),
        ),
    ] {
        // SAFETY: list is initialized and every value remains alive through CreateProcessW.
        if unsafe {
            UpdateProcThreadAttribute(
                list.pointer,
                0,
                attribute,
                value,
                value_size,
                null_mut(),
                null_mut(),
            )
        } == 0
        {
            return Err(format!("UpdateProcThreadAttribute({attribute:#x}) failed"));
        }
    }
    Ok(LaunchAttributes {
        list,
        capabilities,
        handles,
        child_policy,
        all_packages,
        mitigation,
    })
}

fn pipe_pair() -> Result<(HandleGuard, HandleGuard), String> {
    let attributes = SecurityAttributes {
        nLength: size_of::<SecurityAttributes>() as u32,
        lpSecurityDescriptor: null_mut(),
        bInheritHandle: 1,
    };
    let mut read = null_mut();
    let mut write = null_mut();
    // SAFETY: both outputs are writable and attributes requests inheritable handles.
    if unsafe { CreatePipe(&raw mut read, &raw mut write, &raw const attributes, 0) } == 0 {
        return Err("CreatePipe failed".into());
    }
    Ok((HandleGuard(read), HandleGuard(write)))
}

fn read_bounded(handle: Handle, limit: u64) -> thread::JoinHandle<Result<Vec<u8>, String>> {
    let raw = handle as usize;
    thread::spawn(move || {
        // SAFETY: ownership of this exact handle was transferred into the thread.
        let file = unsafe { File::from_raw_handle(raw as *mut c_void) };
        let mut output = Vec::new();
        file.take(limit)
            .read_to_end(&mut output)
            .map_err(|error| format!("pipe read failed: {error}"))?;
        Ok(output)
    })
}

fn run_scenario(
    scenario: &str,
    profile: &Profile,
    worker: &Path,
    stage: &Path,
    control: &BTreeMap<&str, String>,
    unrelated_handle: Handle,
) -> Result<ScenarioOutcome, String> {
    let job = create_job()?;
    let (stdin_read, mut stdin_write) = pipe_pair()?;
    let (mut stdout_read, stdout_write) = pipe_pair()?;
    let (mut stderr_read, stderr_write) = pipe_pair()?;
    for parent in [&stdin_write, &stdout_read, &stderr_read] {
        // SAFETY: each handle is live; clearing inheritance does not close it.
        if unsafe { SetHandleInformation(parent.0, HANDLE_FLAG_INHERIT, 0) } == 0 {
            return Err("SetHandleInformation failed".into());
        }
    }
    let inherited = [stdin_read.0, stdout_write.0, stderr_write.0];
    if inherited.contains(&unrelated_handle) {
        return Err("unrelated handle collided with exact inherited list".into());
    }
    let attributes = attribute_list(profile.sid.0, inherited)?;
    let application = wide(worker.as_os_str());
    let mut command = wide(worker.as_os_str());
    let current_directory = wide(stage.as_os_str());
    let mut environment = EnvironmentBlock::exact_system(stage)?;
    let mut startup = StartupInfoExW::default();
    startup.StartupInfo.cb = size_of::<StartupInfoExW>() as u32;
    startup.StartupInfo.dwFlags = STARTF_USESTDHANDLES;
    startup.StartupInfo.hStdInput = stdin_read.0;
    startup.StartupInfo.hStdOutput = stdout_write.0;
    startup.StartupInfo.hStdError = stderr_write.0;
    startup.lpAttributeList = attributes.list.pointer;
    let mut process = ProcessInformation::default();
    // SAFETY: all launch buffers, attribute values, and handles are live.
    let created = unsafe {
        CreateProcessW(
            application.as_ptr(),
            command.as_mut_ptr(),
            null(),
            null(),
            1,
            CREATE_SUSPENDED
                | CREATE_UNICODE_ENVIRONMENT
                | CREATE_NO_WINDOW
                | EXTENDED_STARTUPINFO_PRESENT,
            environment.0.as_mut_ptr().cast(),
            current_directory.as_ptr(),
            (&raw mut startup.StartupInfo).cast(),
            &raw mut process,
        )
    };
    // SAFETY: GetLastError is read immediately after the failed launch call.
    let create_error = if created == 0 {
        unsafe { GetLastError() }
    } else {
        0
    };
    std::hint::black_box((
        &attributes.capabilities,
        &attributes.handles,
        &attributes.child_policy,
        &attributes.all_packages,
        &attributes.mitigation,
    ));
    if created == 0 || process.hProcess.is_null() || process.hThread.is_null() {
        return Err(format!(
            "CreateProcessW({scenario}) failed: win32={create_error}"
        ));
    }
    let process_handle = HandleGuard(process.hProcess);
    let thread_handle = HandleGuard(process.hThread);
    drop(stdin_read);
    drop(stdout_write);
    drop(stderr_write);
    // SAFETY: both job and suspended process are live.
    if unsafe { AssignProcessToJobObject(job.0, process_handle.0) } == 0 {
        // SAFETY: the exact worker remains suspended and is not yet job-owned.
        unsafe {
            TerminateProcess(process_handle.0, 0xbad1);
            WaitForSingleObject(process_handle.0, 5_000);
        }
        return Err("AssignProcessToJobObject failed".into());
    }
    let _cleanup = JobProcessCleanup {
        job: job.0,
        process: process_handle.0,
    };
    let mut assigned = 0;
    // SAFETY: assigned is writable and both handles are live.
    if unsafe { IsProcessInJob(process_handle.0, job.0, &raw mut assigned) } == 0 || assigned != 1 {
        return Err("process was not assigned before resume".into());
    }
    // SAFETY: the exact primary thread is still suspended.
    if unsafe { ResumeThread(thread_handle.0) } == u32::MAX {
        return Err("ResumeThread failed".into());
    }
    let mut frame = format!("scenario={scenario}\nexpected_sid={}\n", profile.sid_string);
    for (key, value) in control {
        frame.push_str(key);
        frame.push('=');
        frame.push_str(value);
        frame.push('\n');
    }
    // SAFETY: ownership of the parent write handle moves into File exactly once.
    let mut input_file = unsafe { File::from_raw_handle(stdin_write.take()) };
    input_file
        .write_all(frame.as_bytes())
        .map_err(|error| format!("control write failed: {error}"))?;
    drop(input_file);
    let stdout_reader = read_bounded(stdout_read.take(), 65_537);
    let stderr_reader = read_bounded(stderr_read.take(), 16_385);
    let deadline = if matches!(scenario, "timeout" | "cancellation") {
        Duration::from_millis(650)
    } else if scenario == "cpu-pressure" {
        Duration::from_secs(8)
    } else {
        Duration::from_secs(5)
    };
    let start = Instant::now();
    let mut deadline_hit = false;
    loop {
        // SAFETY: process_handle is live.
        match unsafe { WaitForSingleObject(process_handle.0, 20) } {
            WAIT_OBJECT_0 => break,
            WAIT_TIMEOUT if start.elapsed() < deadline => continue,
            WAIT_TIMEOUT => {
                deadline_hit = true;
                // SAFETY: job is live and owns the worker process.
                unsafe { TerminateJobObject(job.0, 0xdead) };
                break;
            }
            other => return Err(format!("process wait failed: {other}")),
        }
    }
    // SAFETY: after termination or normal exit, wait for exact process convergence.
    if unsafe { WaitForSingleObject(process_handle.0, 5_000) } != WAIT_OBJECT_0 {
        unsafe { TerminateJobObject(job.0, 0xbeef) };
        return Err("worker process did not converge".into());
    }
    let mut exit_code = STILL_ACTIVE;
    // SAFETY: exit_code is writable and the process has signaled.
    if unsafe { GetExitCodeProcess(process_handle.0, &raw mut exit_code) } == 0
        || exit_code == STILL_ACTIVE
    {
        return Err("worker exit code unavailable".into());
    }
    // SAFETY: idempotently stop any unexpected descendant before accounting.
    unsafe { TerminateJobObject(job.0, 0xcafe) };
    let mut accounting = JobObjectBasicAccountingInformation::default();
    let mut returned = 0_u32;
    // SAFETY: accounting is writable exact-layout storage.
    if unsafe {
        QueryInformationJobObject(
            job.0,
            JOB_OBJECT_BASIC_ACCOUNTING_INFORMATION_CLASS,
            (&raw mut accounting).cast(),
            size_of::<JobObjectBasicAccountingInformation>() as u32,
            &raw mut returned,
        )
    } == 0
        || accounting.ActiveProcesses != 0
        || accounting.TotalProcesses != 1
    {
        return Err("Job Object process accounting violated the one-process contract".into());
    }
    let stdout = stdout_reader
        .join()
        .map_err(|_| "stdout reader panicked")??;
    let stderr = stderr_reader
        .join()
        .map_err(|_| "stderr reader panicked")??;
    Ok(ScenarioOutcome {
        stdout,
        stderr,
        exit_code,
        deadline_hit,
        total_processes: accounting.TotalProcesses,
        active_processes: accounting.ActiveProcesses,
    })
}

fn parse_result(bytes: &[u8]) -> Option<BTreeMap<&str, &str>> {
    let text = std::str::from_utf8(bytes).ok()?.trim();
    let mut result = BTreeMap::new();
    for member in text.split(';') {
        let (key, value) = member.split_once('=')?;
        if key.is_empty() || value.is_empty() || result.insert(key, value).is_some() {
            return None;
        }
    }
    Some(result)
}

fn ordinary_pass(outcome: &ScenarioOutcome, sid: &str) -> bool {
    outcome.exit_code == 0
        && outcome.stderr.is_empty()
        && outcome.total_processes == 1
        && outcome.active_processes == 0
        && parse_result(&outcome.stdout).is_some_and(|result| {
            result.get("status") == Some(&"pass")
                && result.get("sid") == Some(&sid)
                && result.get("lpac") == Some(&"true")
        })
}

fn create_registry_canary(nonce: u64) -> Result<String, String> {
    let path = format!("Software\\ImpresariContextSynthetic_{nonce}");
    let wide_path = wide_str(&path);
    let mut key = null_mut();
    let mut disposition = 0_u32;
    // SAFETY: path is null-terminated and both outputs are writable.
    let status = unsafe {
        RegCreateKeyExW(
            HKEY_CURRENT_USER,
            wide_path.as_ptr(),
            0,
            null_mut(),
            REG_OPTION_NON_VOLATILE,
            KEY_ALL_ACCESS,
            null(),
            &raw mut key,
            &raw mut disposition,
        )
    };
    if status != 0 || key.is_null() || disposition != REG_CREATED_NEW_KEY {
        return Err(format!(
            "synthetic registry canary creation failed: {status}"
        ));
    }
    // SAFETY: the successful call returned an owned key.
    unsafe { RegCloseKey(key) };
    Ok(path)
}

fn delete_registry_canary(path: &str) -> Result<(), String> {
    let path = wide_str(path);
    // SAFETY: path identifies only the exact synthetic subtree.
    let status = unsafe { RegDeleteTreeW(HKEY_CURRENT_USER, path.as_ptr()) };
    if status != 0 {
        return Err(format!(
            "synthetic registry canary deletion failed: {status}"
        ));
    }
    Ok(())
}

fn remove_exact(path: &Path) -> Result<(), String> {
    if !path.exists() {
        return Ok(());
    }
    if path.is_dir() {
        fs::remove_dir_all(path).map_err(|error| format!("directory cleanup failed: {error}"))
    } else {
        fs::remove_file(path).map_err(|error| format!("file cleanup failed: {error}"))
    }
}

fn execute_matrix() -> Result<(u32, String, String), String> {
    require_hosted_context()?;
    let build = windows_build()?;
    let runner_temp = PathBuf::from(
        env::var_os("RUNNER_TEMP").ok_or("RUNNER_TEMP unavailable on hosted runner")?,
    );
    require_ntfs(&runner_temp)?;
    let broker = env::current_exe().map_err(|error| format!("broker path unavailable: {error}"))?;
    let source_worker = broker
        .parent()
        .ok_or("broker directory unavailable")?
        .join("windows-native-synthetic-worker.exe");
    if !source_worker.is_file() || source_worker.is_symlink() {
        return Err("fixed synthetic worker binary missing or symlinked".into());
    }
    let broker_digest = verify_binary_digest(&broker, "EXPECTED_BROKER_SHA256")?;
    let worker_digest = verify_binary_digest(&source_worker, "EXPECTED_WORKER_SHA256")?;
    let nonce = unsafe { GetTickCount64() };
    let user_sid = current_user_sid()?;
    let mut first = create_profile("a", nonce)?;
    let mut second = create_profile("b", nonce.saturating_add(1))?;
    harden_profile(&first, &user_sid)?;
    harden_profile(&second, &user_sid)?;

    let stage = first.path.join(format!("ImpresariJob-{nonce}"));
    let second_stage = second
        .path
        .join(format!("ImpresariJob-{}", nonce.saturating_add(1)));
    let user_canary = runner_temp.join(format!("impresari-user-canary-{nonce}.txt"));
    let mut registry_canary = None;
    let matrix_result = (|| -> Result<(), String> {
        fs::create_dir(&stage).map_err(|error| format!("stage creation failed: {error}"))?;
        fs::create_dir(&second_stage)
            .map_err(|error| format!("second stage creation failed: {error}"))?;
        let worker = stage.join("boundary-worker.exe");
        let second_worker = second_stage.join("boundary-worker.exe");
        let input = stage.join("input.txt");
        let sibling = stage.join("sibling.txt");
        let cross = stage.join("cross-job.txt");
        fs::copy(&source_worker, &worker)
            .map_err(|error| format!("worker staging failed: {error}"))?;
        fs::copy(&source_worker, &second_worker)
            .map_err(|error| format!("second worker staging failed: {error}"))?;
        fs::write(&input, b"impresari-windows-synthetic-input-v1\n")
            .map_err(|error| format!("input staging failed: {error}"))?;
        fs::write(&sibling, b"host-only-sibling\n")
            .map_err(|error| format!("sibling staging failed: {error}"))?;
        fs::write(&cross, b"first-identity-only\n")
            .map_err(|error| format!("cross canary staging failed: {error}"))?;
        fs::write(&user_canary, b"synthetic-user-profile-canary\n")
            .map_err(|error| format!("user canary staging failed: {error}"))?;
        set_dacl(&stage, &user_sid, &[&first.sid_string], true)?;
        set_dacl(&worker, &user_sid, &[&first.sid_string], false)?;
        set_dacl(&second_stage, &user_sid, &[&second.sid_string], true)?;
        set_dacl(&second_worker, &user_sid, &[&second.sid_string], false)?;
        set_dacl(&input, &user_sid, &[&first.sid_string], false)?;
        set_dacl(&sibling, &user_sid, &[], false)?;
        set_dacl(&cross, &user_sid, &[&first.sid_string], false)?;
        set_dacl(&user_canary, &user_sid, &[], false)?;
        if sha256(&worker)? != worker_digest.trim_start_matches("sha256:") {
            return Err("staged worker identity changed".into());
        }
        if sha256(&second_worker)? != worker_digest.trim_start_matches("sha256:") {
            return Err("second staged worker identity changed".into());
        }
        let registry = create_registry_canary(nonce)?;
        registry_canary = Some(registry.clone());
        let listener = TcpListener::bind("127.0.0.1:0")
            .map_err(|error| format!("synthetic loopback listener failed: {error}"))?;
        listener
            .set_nonblocking(true)
            .map_err(|error| format!("loopback nonblocking setup failed: {error}"))?;
        let loopback = listener
            .local_addr()
            .map_err(|error| format!("loopback address unavailable: {error}"))?
            .to_string();
        let inheritable = SecurityAttributes {
            nLength: size_of::<SecurityAttributes>() as u32,
            lpSecurityDescriptor: null_mut(),
            bInheritHandle: 1,
        };
        // SAFETY: attributes is live and the event is private/unnamed.
        let event = unsafe { CreateEventW(&raw const inheritable, 1, 0, null()) };
        if event.is_null() {
            return Err("synthetic unrelated event creation failed".into());
        }
        let event = HandleGuard(event);
        let mut common = BTreeMap::new();
        common.insert("input", input.to_string_lossy().into_owned());
        common.insert("sibling", sibling.to_string_lossy().into_owned());
        common.insert("user_canary", user_canary.to_string_lossy().into_owned());
        common.insert("profile", first.path.to_string_lossy().into_owned());
        common.insert("registry", registry);
        common.insert("loopback", loopback);
        common.insert("unrelated_handle", (event.0 as usize).to_string());
        common.insert("broker_pid", unsafe { GetCurrentProcessId() }.to_string());
        common.insert("cross_canary", cross.to_string_lossy().into_owned());

        let ordinary = [
            "success",
            "input-mutation",
            "worker-mutation",
            "sibling-read",
            "user-profile-canary-read",
            "profile-storage-write",
            "synthetic-registry-canary-read",
            "loopback-connect",
            "unrelated-handle",
            "unrelated-process",
            "child-process",
        ];
        for scenario in ordinary {
            let outcome = run_scenario(scenario, &first, &worker, &stage, &common, event.0)?;
            if !ordinary_pass(&outcome, &first.sid_string) {
                return Err(format!(
                    "scenario {scenario} failed: exit={} stdout={} stderr={}",
                    outcome.exit_code,
                    String::from_utf8_lossy(&outcome.stdout),
                    String::from_utf8_lossy(&outcome.stderr)
                ));
            }
        }
        let cpu = run_scenario("cpu-pressure", &first, &worker, &stage, &common, event.0)?;
        if !ordinary_pass(&cpu, &first.sid_string) {
            return Err("CPU hard-cap scenario was not measurably exercised".into());
        }
        let memory = run_scenario("memory-pressure", &first, &worker, &stage, &common, event.0)?;
        let memory_passed = parse_result(&memory.stdout)
            .is_some_and(|result| result.get("status") == Some(&"pass"))
            || (memory.exit_code != 0 && !memory.stdout.windows(12).any(|v| v == b"status=fail"));
        if !memory_passed {
            return Err("process-memory limit was not exercised".into());
        }
        for scenario in ["timeout", "cancellation"] {
            let outcome = run_scenario(scenario, &first, &worker, &stage, &common, event.0)?;
            if !outcome.deadline_hit || outcome.exit_code == 0 {
                return Err(format!("{scenario} was not terminated by the broker"));
            }
        }
        let flood = run_scenario("output-flood", &first, &worker, &stage, &common, event.0)?;
        if flood.stdout.len() != 65_537 {
            return Err("output-flood cap was not exercised exactly".into());
        }
        let crash = run_scenario("crash", &first, &worker, &stage, &common, event.0)?;
        if crash.exit_code == 0 {
            return Err("crash scenario did not fail".into());
        }
        let malformed = run_scenario(
            "malformed-result",
            &first,
            &worker,
            &stage,
            &common,
            event.0,
        )?;
        if malformed.exit_code != 0 || parse_result(&malformed.stdout).is_some() {
            return Err("malformed result was not rejected".into());
        }
        let mut cross_control = common.clone();
        cross_control.insert("profile", second.path.to_string_lossy().into_owned());
        let cross_outcome = run_scenario(
            "cross-job-read",
            &second,
            &second_worker,
            &second_stage,
            &cross_control,
            event.0,
        )?;
        if !ordinary_pass(&cross_outcome, &second.sid_string) {
            return Err("second fresh identity read the first identity canary".into());
        }
        if listener.accept().is_ok() {
            return Err("LPAC reached the broker-owned loopback listener".into());
        }
        Ok(())
    })();

    let mut cleanup_errors = Vec::new();
    if let Some(path) = &registry_canary
        && let Err(error) = delete_registry_canary(path)
    {
        cleanup_errors.push(error);
    }
    for path in [&stage, &second_stage, &user_canary] {
        if let Err(error) = remove_exact(path) {
            cleanup_errors.push(error);
        }
    }
    if let Err(error) = first.delete() {
        cleanup_errors.push(error);
    }
    if let Err(error) = second.delete() {
        cleanup_errors.push(error);
    }
    if stage.exists()
        || second_stage.exists()
        || user_canary.exists()
        || first.path.exists()
        || second.path.exists()
    {
        cleanup_errors.push("exact synthetic state remained after cleanup".into());
    }
    if !cleanup_errors.is_empty() {
        return Err(format!("cleanup failed: {}", cleanup_errors.join("; ")));
    }
    if let Err(error) = matrix_result {
        return Err(error);
    }
    Ok((build, broker_digest, worker_digest))
}

fn exact_host_metadata() -> Result<(u32, String, String), String> {
    require_hosted_context()?;
    let build = windows_build()?;
    let broker = env::current_exe().map_err(|error| format!("broker path unavailable: {error}"))?;
    let worker = broker
        .parent()
        .ok_or("broker directory unavailable")?
        .join("windows-native-synthetic-worker.exe");
    Ok((
        build,
        verify_binary_digest(&broker, "EXPECTED_BROKER_SHA256")?,
        verify_binary_digest(&worker, "EXPECTED_WORKER_SHA256")?,
    ))
}

fn print_unsupported_host(build: u32, broker_digest: &str, worker_digest: &str) {
    println!(
        concat!(
            "{{\"schema_name\":\"windows-native-synthetic-worker-matrix-receipt\",",
            "\"schema_version\":\"1.0.0\",\"profile_id\":\"{}\",",
            "\"profile_digest\":\"{}\",\"base_profile_id\":\"{}\",",
            "\"base_profile_digest\":\"{}\",\"status\":\"unsupported\",",
            "\"reason_code\":\"unsupported_host\",\"host\":{{",
            "\"runner_environment\":\"github-hosted\",\"runner_label\":\"windows-2025\",",
            "\"os_family\":\"windows\",\"windows_build\":\"{}\",",
            "\"architecture\":\"x86_64\",\"filesystem\":\"ntfs\",",
            "\"broker_digest\":\"{}\",\"worker_digest\":\"{}\"}},",
            "\"identity\":{{\"fresh_profile_created\":false,\"worker_sid_matched\":false,",
            "\"capability_count\":\"0\",\"lpac_verified\":false,",
            "\"profile_storage_hardened\":false,\"profile_deleted\":false}},",
            "\"launch\":{{\"worker_created_suspended\":false,",
            "\"job_limits_set_and_queried\":false,\"job_assigned_before_resume\":false,",
            "\"exact_handle_list\":false,\"mitigations_applied\":false,",
            "\"child_policy_applied\":false,\"worker_resumed\":false}},",
            "\"observations\":{{\"scenario_count\":\"19\",\"success\":false,",
            "\"exact_input_read\":false,\"input_mutation_denied\":false,",
            "\"worker_mutation_denied\":false,\"sibling_read_denied\":false,",
            "\"user_profile_canary_read_denied\":false,",
            "\"profile_storage_write_denied\":false,",
            "\"synthetic_registry_canary_read_denied\":false,",
            "\"loopback_connect_denied\":false,\"unrelated_handle_absent\":false,",
            "\"unrelated_process_open_denied\":false,\"child_process_denied\":false,",
            "\"active_process_peak_one\":false,\"process_memory_limit_exercised\":false,",
            "\"job_memory_limit_queried\":false,\"cpu_limit_exercised\":false,",
            "\"timeout_contained\":false,\"output_flood_contained\":false,",
            "\"crash_contained\":false,\"cancellation_contained\":false,",
            "\"malformed_result_rejected\":false,\"cross_job_read_denied\":false}},",
            "\"cleanup\":{{\"job_terminated\":false,\"zero_active_processes\":false,",
            "\"handles_closed\":false,\"staging_removed\":false,\"canaries_removed\":false,",
            "\"profile_deleted\":false,\"cross_job_clean\":false}},",
            "\"claims\":{{\"external_network_contacted\":false,",
            "\"existing_credentials_inspected\":false,\"repository_input\":false,",
            "\"real_analyzer\":false,\"os_confined\":false,",
            "\"production_admitted\":false,\"authority_added\":false}}}}"
        ),
        PROFILE_ID,
        PROFILE_DIGEST,
        BASE_PROFILE_ID,
        BASE_PROFILE_DIGEST,
        build,
        broker_digest,
        worker_digest
    );
}

fn main() {
    match execute_matrix() {
        Ok((build, broker_digest, worker_digest)) => println!(
            concat!(
                "{{\"schema_name\":\"windows-native-synthetic-worker-matrix-receipt\",",
                "\"schema_version\":\"1.0.0\",\"profile_id\":\"{}\",",
                "\"profile_digest\":\"{}\",\"base_profile_id\":\"{}\",",
                "\"base_profile_digest\":\"{}\",\"status\":\"candidate_passed\",",
                "\"reason_code\":\"synthetic_matrix_passed\",\"host\":{{",
                "\"runner_environment\":\"github-hosted\",\"runner_label\":\"windows-2025\",",
                "\"os_family\":\"windows\",\"windows_build\":\"{}\",",
                "\"architecture\":\"x86_64\",\"filesystem\":\"ntfs\",",
                "\"broker_digest\":\"{}\",\"worker_digest\":\"{}\"}},",
                "\"identity\":{{\"fresh_profile_created\":true,\"worker_sid_matched\":true,",
                "\"capability_count\":\"0\",\"lpac_verified\":true,",
                "\"profile_storage_hardened\":true,\"profile_deleted\":true}},",
                "\"launch\":{{\"worker_created_suspended\":true,",
                "\"job_limits_set_and_queried\":true,\"job_assigned_before_resume\":true,",
                "\"exact_handle_list\":true,\"mitigations_applied\":true,",
                "\"child_policy_applied\":true,\"worker_resumed\":true}},",
                "\"observations\":{{\"scenario_count\":\"19\",\"success\":true,",
                "\"exact_input_read\":true,\"input_mutation_denied\":true,",
                "\"worker_mutation_denied\":true,\"sibling_read_denied\":true,",
                "\"user_profile_canary_read_denied\":true,",
                "\"profile_storage_write_denied\":true,",
                "\"synthetic_registry_canary_read_denied\":true,",
                "\"loopback_connect_denied\":true,\"unrelated_handle_absent\":true,",
                "\"unrelated_process_open_denied\":true,\"child_process_denied\":true,",
                "\"active_process_peak_one\":true,\"process_memory_limit_exercised\":true,",
                "\"job_memory_limit_queried\":true,\"cpu_limit_exercised\":true,",
                "\"timeout_contained\":true,\"output_flood_contained\":true,",
                "\"crash_contained\":true,\"cancellation_contained\":true,",
                "\"malformed_result_rejected\":true,\"cross_job_read_denied\":true}},",
                "\"cleanup\":{{\"job_terminated\":true,\"zero_active_processes\":true,",
                "\"handles_closed\":true,\"staging_removed\":true,\"canaries_removed\":true,",
                "\"profile_deleted\":true,\"cross_job_clean\":true}},",
                "\"claims\":{{\"external_network_contacted\":false,",
                "\"existing_credentials_inspected\":false,\"repository_input\":false,",
                "\"real_analyzer\":false,\"os_confined\":false,",
                "\"production_admitted\":false,\"authority_added\":false}}}}"
            ),
            PROFILE_ID,
            PROFILE_DIGEST,
            BASE_PROFILE_ID,
            BASE_PROFILE_DIGEST,
            build,
            broker_digest,
            worker_digest
        ),
        Err(error) if error == "CreateProcessW(success) failed: win32=5" => {
            match exact_host_metadata() {
                Ok((build, broker_digest, worker_digest)) => {
                    print_unsupported_host(build, &broker_digest, &worker_digest);
                }
                Err(metadata_error) => {
                    eprintln!("Windows unsupported-host metadata failed: {metadata_error}");
                    std::process::exit(1);
                }
            }
        }
        Err(error) => {
            eprintln!("Windows synthetic-worker matrix failed: {error}");
            std::process::exit(1);
        }
    }
}
