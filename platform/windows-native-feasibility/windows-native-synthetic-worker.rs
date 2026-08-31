// SPDX-License-Identifier: Apache-2.0
#![cfg(windows)]
#![deny(unsafe_op_in_unsafe_fn)]
#![allow(non_snake_case)]

use std::{
    collections::BTreeMap,
    env,
    ffi::{OsStr, c_void},
    fs::{self, OpenOptions},
    io::{self, Read, Write},
    mem::{size_of, zeroed},
    net::{SocketAddr, TcpStream},
    os::windows::ffi::OsStrExt,
    path::Path,
    ptr::{null, null_mut},
    thread,
    time::{Duration, Instant},
};

type Handle = *mut c_void;
type HKey = *mut c_void;
type Sid = *mut c_void;

const TOKEN_QUERY: u32 = 0x0008;
const TOKEN_IS_APP_CONTAINER: u32 = 29;
const TOKEN_APP_CONTAINER_SID: u32 = 31;
const TOKEN_IS_LESS_PRIVILEGED_APP_CONTAINER: u32 = 46;
const PROCESS_QUERY_LIMITED_INFORMATION: u32 = 0x1000;
const WAIT_FAILED: u32 = 0xffff_ffff;
const KEY_READ: u32 = 0x0002_0019;
const HKEY_CURRENT_USER: HKey = 0x8000_0001_usize as HKey;
const CREATE_UNICODE_ENVIRONMENT: u32 = 0x0000_0400;

#[repr(C)]
#[derive(Clone, Copy)]
struct TokenAppContainerInformation {
    TokenAppContainer: Sid,
}

#[repr(C)]
#[derive(Clone, Copy, Default)]
struct FileTime {
    dwLowDateTime: u32,
    dwHighDateTime: u32,
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

#[link(name = "kernel32")]
unsafe extern "system" {
    fn GetCurrentProcess() -> Handle;
    fn GetProcessTimes(
        process: Handle,
        creation: *mut FileTime,
        exit: *mut FileTime,
        kernel: *mut FileTime,
        user: *mut FileTime,
    ) -> i32;
    fn OpenProcess(access: u32, inherit: i32, process_id: u32) -> Handle;
    fn WaitForSingleObject(handle: Handle, milliseconds: u32) -> u32;
    fn CloseHandle(handle: Handle) -> i32;
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
    fn VirtualAlloc(
        address: *mut c_void,
        size: usize,
        allocation_type: u32,
        protect: u32,
    ) -> *mut c_void;
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
    fn RegOpenKeyExW(
        key: HKey,
        subkey: *const u16,
        options: u32,
        access: u32,
        result: *mut HKey,
    ) -> i32;
    fn RegCloseKey(key: HKey) -> i32;
}

#[link(name = "kernel32")]
unsafe extern "system" {
    fn LocalFree(memory: *mut c_void) -> *mut c_void;
}

struct HandleGuard(Handle);

impl Drop for HandleGuard {
    fn drop(&mut self) {
        if !self.0.is_null() {
            // SAFETY: this guard owns one live kernel handle.
            unsafe { CloseHandle(self.0) };
        }
    }
}

fn wide(value: &OsStr) -> Vec<u16> {
    value.encode_wide().chain(Some(0)).collect()
}

fn wide_str(value: &str) -> Vec<u16> {
    wide(OsStr::new(value))
}

fn parse_control() -> Result<BTreeMap<String, String>, String> {
    let mut input = io::stdin().take(16_385);
    let mut bytes = Vec::new();
    input
        .read_to_end(&mut bytes)
        .map_err(|error| format!("control read failed: {error}"))?;
    if bytes.len() > 16_384 {
        return Err("control frame exceeded 16384 bytes".into());
    }
    let text = String::from_utf8(bytes).map_err(|_| "control frame was not UTF-8")?;
    let mut values = BTreeMap::new();
    for line in text.lines() {
        let (key, value) = line
            .split_once('=')
            .ok_or_else(|| "control line missing separator".to_owned())?;
        if key.is_empty()
            || value.contains(['\r', '\n', '\0'])
            || values.insert(key.to_owned(), value.to_owned()).is_some()
        {
            return Err("control frame was not canonical".into());
        }
    }
    Ok(values)
}

fn required<'a>(control: &'a BTreeMap<String, String>, key: &str) -> Result<&'a str, String> {
    control
        .get(key)
        .map(String::as_str)
        .ok_or_else(|| format!("missing control field: {key}"))
}

fn token_u32(token: Handle, class: u32) -> Result<u32, String> {
    let mut value = 0_u32;
    let mut returned = 0_u32;
    // SAFETY: value is writable storage of the exact requested DWORD size.
    let ok = unsafe {
        GetTokenInformation(
            token,
            class,
            (&raw mut value).cast(),
            size_of::<u32>() as u32,
            &raw mut returned,
        )
    };
    if ok == 0 || returned != size_of::<u32>() as u32 {
        return Err(format!("GetTokenInformation({class}) failed"));
    }
    Ok(value)
}

fn current_identity() -> Result<(String, bool), String> {
    let mut token = null_mut();
    // SAFETY: token is writable and the current-process pseudo-handle is valid.
    if unsafe { OpenProcessToken(GetCurrentProcess(), TOKEN_QUERY, &raw mut token) } == 0 {
        return Err("OpenProcessToken failed".into());
    }
    let token = HandleGuard(token);
    if token_u32(token.0, TOKEN_IS_APP_CONTAINER)? != 1 {
        return Err("worker token is not AppContainer".into());
    }
    let lpac = token_u32(token.0, TOKEN_IS_LESS_PRIVILEGED_APP_CONTAINER)? == 1;
    let mut information = TokenAppContainerInformation {
        TokenAppContainer: null_mut(),
    };
    let mut returned = 0_u32;
    // SAFETY: information is writable storage with the documented layout.
    let ok = unsafe {
        GetTokenInformation(
            token.0,
            TOKEN_APP_CONTAINER_SID,
            (&raw mut information).cast(),
            size_of::<TokenAppContainerInformation>() as u32,
            &raw mut returned,
        )
    };
    if ok == 0 || information.TokenAppContainer.is_null() {
        return Err("worker AppContainer SID unavailable".into());
    }
    let mut raw = null_mut();
    // SAFETY: the token owns the live SID and raw is writable.
    if unsafe { ConvertSidToStringSidW(information.TokenAppContainer, &raw mut raw) } == 0
        || raw.is_null()
    {
        return Err("ConvertSidToStringSidW failed".into());
    }
    let mut length = 0_usize;
    // SAFETY: raw points to a null-terminated string allocated by LocalAlloc.
    unsafe {
        while *raw.add(length) != 0 {
            length += 1;
        }
    }
    // SAFETY: the preceding scan found the terminator inside the API allocation.
    let slice = unsafe { std::slice::from_raw_parts(raw, length) };
    let sid = String::from_utf16(slice).map_err(|_| "SID was not valid UTF-16")?;
    // SAFETY: raw was allocated for ConvertSidToStringSidW and is no longer used.
    unsafe { LocalFree(raw.cast()) };
    Ok((sid, lpac))
}

fn file_denied(path: &str, write: bool) -> bool {
    if write {
        OpenOptions::new().write(true).open(path).is_err()
    } else {
        fs::File::open(path).is_err()
    }
}

fn create_denied(path: &Path) -> bool {
    OpenOptions::new()
        .create_new(true)
        .write(true)
        .open(path)
        .is_err()
}

fn registry_denied(path: &str) -> bool {
    let path = wide_str(path);
    let mut key = null_mut();
    // SAFETY: path is null-terminated and key is writable.
    let status =
        unsafe { RegOpenKeyExW(HKEY_CURRENT_USER, path.as_ptr(), 0, KEY_READ, &raw mut key) };
    if status == 0 && !key.is_null() {
        // SAFETY: the successful call returned an owned key.
        unsafe { RegCloseKey(key) };
        false
    } else {
        true
    }
}

fn child_denied() -> bool {
    let executable = match env::current_exe() {
        Ok(value) => value,
        Err(_) => return false,
    };
    let application = wide(executable.as_os_str());
    let mut command = wide(executable.as_os_str());
    let mut startup = StartupInfoW {
        cb: size_of::<StartupInfoW>() as u32,
        ..StartupInfoW::default()
    };
    let mut information = ProcessInformation::default();
    let environment = [0_u16, 0_u16];
    // SAFETY: all pointers refer to live buffers for the duration of the call.
    let created = unsafe {
        CreateProcessW(
            application.as_ptr(),
            command.as_mut_ptr(),
            null(),
            null(),
            0,
            CREATE_UNICODE_ENVIRONMENT,
            environment.as_ptr().cast(),
            null(),
            &raw mut startup,
            &raw mut information,
        )
    };
    if created == 0 {
        true
    } else {
        // SAFETY: an unexpected successful launch returned both handles.
        unsafe {
            CloseHandle(information.hThread);
            CloseHandle(information.hProcess);
        }
        false
    }
}

fn filetime_ticks(value: FileTime) -> u64 {
    (u64::from(value.dwHighDateTime) << 32) | u64::from(value.dwLowDateTime)
}

fn cpu_pressure() -> Result<(u64, u128), String> {
    let mut creation = FileTime::default();
    let mut exit = FileTime::default();
    let mut kernel_before = FileTime::default();
    let mut user_before = FileTime::default();
    // SAFETY: all outputs are writable FILETIME values.
    if unsafe {
        GetProcessTimes(
            GetCurrentProcess(),
            &raw mut creation,
            &raw mut exit,
            &raw mut kernel_before,
            &raw mut user_before,
        )
    } == 0
    {
        return Err("GetProcessTimes(before) failed".into());
    }
    let start = Instant::now();
    let mut state = 0x9e37_79b9_u64;
    while start.elapsed() < Duration::from_millis(1_600) {
        state = state.rotate_left(7) ^ 0xa5a5_5a5a_d3c4_b2e1;
        std::hint::black_box(state);
    }
    let wall = start.elapsed().as_millis();
    let mut kernel_after = FileTime::default();
    let mut user_after = FileTime::default();
    // SAFETY: all outputs are writable FILETIME values.
    if unsafe {
        GetProcessTimes(
            GetCurrentProcess(),
            &raw mut creation,
            &raw mut exit,
            &raw mut kernel_after,
            &raw mut user_after,
        )
    } == 0
    {
        return Err("GetProcessTimes(after) failed".into());
    }
    let before = filetime_ticks(kernel_before) + filetime_ticks(user_before);
    let after = filetime_ticks(kernel_after) + filetime_ticks(user_after);
    Ok(((after.saturating_sub(before)) / 10_000, wall))
}

fn memory_pressure() -> ! {
    const MEM_COMMIT_RESERVE: u32 = 0x0000_3000;
    const PAGE_READWRITE: u32 = 0x04;
    let mut allocated = 0_usize;
    loop {
        // SAFETY: a null address asks Windows to choose a fresh private region.
        let block =
            unsafe { VirtualAlloc(null_mut(), 1024 * 1024, MEM_COMMIT_RESERVE, PAGE_READWRITE) };
        if block.is_null() {
            println!("status=pass;memory_denied=true;allocated={allocated}");
            std::process::exit(0);
        }
        // SAFETY: VirtualAlloc returned one writable 1 MiB region.
        unsafe { std::ptr::write_bytes(block, 0xa5, 1024 * 1024) };
        allocated += 1024 * 1024;
        if allocated > 96 * 1024 * 1024 {
            println!("status=fail;memory_denied=false;allocated={allocated}");
            std::process::exit(2);
        }
    }
}

fn run(control: &BTreeMap<String, String>) -> Result<String, String> {
    let scenario = required(control, "scenario")?;
    let (sid, lpac) = current_identity()?;
    if required(control, "expected_sid")? != sid || !lpac {
        return Err("worker identity did not match the exact LPAC SID".into());
    }
    let passed = match scenario {
        "success" => fs::read_to_string(required(control, "input")?)
            .map(|value| value == "impresari-windows-synthetic-input-v1\n")
            .unwrap_or(false),
        "input-mutation" => file_denied(required(control, "input")?, true),
        "worker-mutation" => env::current_exe()
            .ok()
            .and_then(|path| OpenOptions::new().write(true).open(path).err())
            .is_some(),
        "sibling-read" => file_denied(required(control, "sibling")?, false),
        "user-profile-canary-read" => file_denied(required(control, "user_canary")?, false),
        "profile-storage-write" => create_denied(
            &Path::new(required(control, "profile")?).join("impresari-write-denial.tmp"),
        ),
        "synthetic-registry-canary-read" => registry_denied(required(control, "registry")?),
        "loopback-connect" => required(control, "loopback")?
            .parse::<SocketAddr>()
            .ok()
            .is_some_and(|address| {
                TcpStream::connect_timeout(&address, Duration::from_millis(600)).is_err()
            }),
        "unrelated-handle" => required(control, "unrelated_handle")?
            .parse::<usize>()
            .ok()
            .is_some_and(|raw| {
                // SAFETY: the numeric value is intentionally not inherited; no
                // object is mutated and WAIT_FAILED proves it is absent.
                (unsafe { WaitForSingleObject(raw as Handle, 0) }) == WAIT_FAILED
            }),
        "unrelated-process" => required(control, "broker_pid")?
            .parse::<u32>()
            .ok()
            .is_some_and(|pid| {
                // SAFETY: the PID identifies only the synthetic broker process.
                let process = unsafe { OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, 0, pid) };
                if process.is_null() {
                    true
                } else {
                    // SAFETY: the successful call returned an owned handle.
                    unsafe { CloseHandle(process) };
                    false
                }
            }),
        "child-process" => child_denied(),
        "cpu-pressure" => {
            let (cpu_ms, wall_ms) = cpu_pressure()?;
            return Ok(format!(
                "status={};sid={sid};lpac=true;cpu_ms={cpu_ms};wall_ms={wall_ms}",
                if wall_ms >= 1_400 && cpu_ms >= 40 && u128::from(cpu_ms) * 2 < wall_ms {
                    "pass"
                } else {
                    "fail"
                }
            ));
        }
        "memory-pressure" => memory_pressure(),
        "timeout" | "cancellation" => {
            thread::sleep(Duration::from_secs(60));
            false
        }
        "output-flood" => {
            let block = [b'x'; 4096];
            for _ in 0..64 {
                io::stdout()
                    .write_all(&block)
                    .map_err(|error| format!("output flood failed: {error}"))?;
            }
            false
        }
        "crash" => std::process::abort(),
        "malformed-result" => return Ok("this is deliberately not a result frame".into()),
        "cross-job-read" => file_denied(required(control, "cross_canary")?, false),
        _ => return Err("unknown scenario".into()),
    };
    Ok(format!(
        "status={};sid={sid};lpac=true",
        if passed { "pass" } else { "fail" }
    ))
}

fn main() {
    let control = match parse_control() {
        Ok(value) => value,
        Err(error) => {
            eprintln!("worker control rejected: {error}");
            std::process::exit(2);
        }
    };
    match run(&control) {
        Ok(result) => println!("{result}"),
        Err(error) => {
            eprintln!("worker scenario failed: {error}");
            std::process::exit(1);
        }
    }
}
