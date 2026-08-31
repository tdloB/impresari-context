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
    ptr::null_mut,
};

const PROFILE_ID: &str = "iar-windows-basecontainer-capability-v1";
const PROFILE_DIGEST: &str =
    "sha256:9f5c8f589cf5f7ce3e6d87b6b7752aeac4da530a81edbb1bf036bf5eb7e84305";
const MINIMUM_WINDOWS_BUILD: u32 = 26_600;
const LOAD_LIBRARY_SEARCH_SYSTEM32: u32 = 0x0000_0800;
const VER_NT_WORKSTATION: u8 = 1;
const VER_NT_DOMAIN_CONTROLLER: u8 = 2;
const VER_NT_SERVER: u8 = 3;

type Module = *mut c_void;

#[repr(C)]
#[derive(Clone, Copy)]
struct OsVersionInfoExW {
    dwOSVersionInfoSize: u32,
    dwMajorVersion: u32,
    dwMinorVersion: u32,
    dwBuildNumber: u32,
    dwPlatformId: u32,
    szCSDVersion: [u16; 128],
    wServicePackMajor: u16,
    wServicePackMinor: u16,
    wSuiteMask: u16,
    wProductType: u8,
    wReserved: u8,
}

#[link(name = "kernel32")]
unsafe extern "system" {
    fn LoadLibraryExW(file_name: *const u16, file: *mut c_void, flags: u32) -> Module;
    fn GetProcAddress(module: Module, procedure_name: *const u8) -> *mut c_void;
    fn FreeLibrary(module: Module) -> i32;
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
    fn RtlGetVersion(version: *mut OsVersionInfoExW) -> i32;
}

fn wide(value: &str) -> Vec<u16> {
    OsStr::new(value).encode_wide().chain(Some(0)).collect()
}

fn require_hosted_context() -> Result<(), String> {
    if env::var("GITHUB_ACTIONS").as_deref() != Ok("true")
        || env::var("RUNNER_ENVIRONMENT").as_deref() != Ok("github-hosted")
        || env::var("EXPECTED_WINDOWS_RUNNER").as_deref() != Ok("windows-11-arm")
    {
        return Err(
            "BaseContainer probe is restricted to the fresh GitHub-hosted windows-11-arm job"
                .into(),
        );
    }
    if env::consts::OS != "windows" || env::consts::ARCH != "aarch64" {
        return Err("unsupported Windows BaseContainer observation target".into());
    }
    Ok(())
}

fn windows_identity() -> Result<(u32, &'static str), String> {
    // SAFETY: zeroed is valid for this plain C structure and its size is set
    // before the documented RtlGetVersion call.
    let mut version: OsVersionInfoExW = unsafe { zeroed() };
    version.dwOSVersionInfoSize = size_of::<OsVersionInfoExW>() as u32;
    // SAFETY: version points to writable storage of the exact declared size.
    let status = unsafe { RtlGetVersion(&mut version) };
    if status != 0 || version.dwBuildNumber == 0 {
        return Err(format!("RtlGetVersion failed: status={status:#010x}"));
    }
    let product_type = match version.wProductType {
        VER_NT_WORKSTATION => "workstation",
        VER_NT_DOMAIN_CONTROLLER => "domain_controller",
        VER_NT_SERVER => "server",
        value => return Err(format!("unsupported Windows product type: {value}")),
    };
    Ok((version.dwBuildNumber, product_type))
}

fn current_volume_filesystem() -> Result<&'static str, String> {
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
    Ok(if name.eq_ignore_ascii_case("NTFS") {
        "ntfs"
    } else {
        "other"
    })
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

fn inspect_system_module() -> Result<(bool, bool, bool), String> {
    let module_name = wide("processmodel.dll");
    // SAFETY: the fixed module name is null-terminated and the search flag
    // restricts resolution to trusted System32.
    let module = unsafe {
        LoadLibraryExW(
            module_name.as_ptr(),
            null_mut(),
            LOAD_LIBRARY_SEARCH_SYSTEM32,
        )
    };
    if module.is_null() {
        return Ok((false, false, false));
    }
    // SAFETY: both byte strings are null-terminated and module remains loaded.
    let create = unsafe {
        !GetProcAddress(module, b"Experimental_CreateProcessInSandbox\0".as_ptr()).is_null()
    };
    // SAFETY: the byte string is null-terminated and module remains loaded.
    let create_as_user = unsafe {
        !GetProcAddress(
            module,
            b"Experimental_CreateProcessAsUserInSandbox\0".as_ptr(),
        )
        .is_null()
    };
    // SAFETY: module is the live handle returned by LoadLibraryExW and is owned here.
    if unsafe { FreeLibrary(module) } == 0 {
        return Err("FreeLibrary failed after trusted export inspection".into());
    }
    Ok((true, create, create_as_user))
}

fn run() -> Result<(u32, &'static str, &'static str, bool, bool, bool), String> {
    require_hosted_context()?;
    let (build, product_type) = windows_identity()?;
    let filesystem = current_volume_filesystem()?;
    let (module, create, create_as_user) = inspect_system_module()?;
    Ok((
        build,
        product_type,
        filesystem,
        module,
        create,
        create_as_user,
    ))
}

fn main() {
    match run() {
        Ok((build, product_type, filesystem, module, create, create_as_user)) => {
            let (status, reason) = if product_type != "workstation" {
                ("unsupported", "unsupported_host_family")
            } else if filesystem != "ntfs" {
                ("unsupported", "unsupported_filesystem")
            } else if build < MINIMUM_WINDOWS_BUILD {
                ("unsupported", "unsupported_build")
            } else if !(module && create && create_as_user) {
                ("unsupported", "unsupported_api_absent")
            } else {
                (
                    "ready_for_basecontainer_rehearsal",
                    "candidate_capability_present",
                )
            };
            println!(
                concat!(
                    "{{\"schema_name\":\"windows-basecontainer-capability-receipt\",",
                    "\"schema_version\":\"1.0.0\",\"profile_id\":\"{}\",",
                    "\"profile_digest\":\"{}\",\"runner_environment\":\"github-hosted\",",
                    "\"runner_label\":\"windows-11-arm\",\"os_family\":\"windows\",",
                    "\"os_product_type\":\"{}\",\"windows_build\":\"{}\",",
                    "\"architecture\":\"arm64\",\"filesystem\":\"{}\",",
                    "\"system_module_inspected\":true,\"processmodel_dll_present\":{},",
                    "\"create_process_in_sandbox_export_present\":{},",
                    "\"create_process_as_user_in_sandbox_export_present\":{},",
                    "\"status\":\"{}\",\"reason_code\":\"{}\",",
                    "\"synthetic_worker_launched\":false,\"appcontainer_profile_created\":false,",
                    "\"host_acl_modified\":false,\"windows_feature_modified\":false,",
                    "\"elevation_requested\":false,\"os_confined\":false,",
                    "\"production_admitted\":false,\"analyzer_execution\":false,",
                    "\"authority_added\":false}}"
                ),
                PROFILE_ID,
                PROFILE_DIGEST,
                product_type,
                build,
                filesystem,
                module,
                create,
                create_as_user,
                status,
                reason
            );
        }
        Err(error) => {
            eprintln!("Windows BaseContainer capability preflight failed: {error}");
            std::process::exit(1);
        }
    }
}
