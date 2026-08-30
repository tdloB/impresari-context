// SPDX-License-Identifier: Apache-2.0
#![forbid(unsafe_code)]
#![doc = "Foreground loopback-only dashboard delivery for Impresari Context."]

mod http;

use std::{
    collections::VecDeque,
    error::Error,
    fmt,
    io::Write as _,
    net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr, TcpListener, TcpStream},
    path::{Path, PathBuf},
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, AtomicUsize, Ordering},
    },
    thread,
    time::{Duration, Instant},
};

use context_dashboard::{
    DashboardError, DashboardErrorCode, DashboardRecord, DashboardSnapshot, LocalBudgetPolicyDraft,
    PolicyStore, PolicyStoreState, build_snapshot, compile_policy, project_event,
};
use context_store::AuditReader;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::http::{Request, Response};

const INDEX_HTML: &[u8] = include_bytes!("../assets/index.html");
const APP_JS: &[u8] = include_bytes!("../assets/app.js");
const APP_CSS: &[u8] = include_bytes!("../assets/app.css");
const VERSION: &str = "1.0.0";
const MAX_SNAPSHOT_BYTES: usize = 524_288;
const MAX_CONNECTIONS: usize = 16;
const MAX_STREAMS: usize = 4;

/// Stable source-free foreground-server failure categories.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DashboardServerErrorCode {
    /// Startup configuration is unsafe or outside the closed profile.
    InvalidConfiguration,
    /// The operating system could not create the loopback listener.
    BindFailure,
    /// The existing audit store was unavailable or incompatible.
    AuditUnavailable,
    /// The exact-owned policy store was unavailable or incompatible.
    PolicyUnavailable,
    /// A bounded request, response, or stream limit was exceeded.
    ResourceLimit,
    /// A malformed or unauthorized HTTP request was rejected.
    Protocol,
    /// An internal source-free operation failed.
    InternalFailure,
}

/// Safe server error that contains no paths, tokens, audit bytes, or policy content.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DashboardServerError {
    code: DashboardServerErrorCode,
}

impl DashboardServerError {
    const fn new(code: DashboardServerErrorCode) -> Self {
        Self { code }
    }

    /// Returns the stable failure category.
    #[must_use]
    pub const fn code(&self) -> DashboardServerErrorCode {
        self.code
    }
}

impl fmt::Display for DashboardServerError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self.code {
            DashboardServerErrorCode::InvalidConfiguration => {
                "invalid local dashboard configuration"
            }
            DashboardServerErrorCode::BindFailure => "local dashboard loopback bind failed",
            DashboardServerErrorCode::AuditUnavailable => {
                "local dashboard audit metadata unavailable"
            }
            DashboardServerErrorCode::PolicyUnavailable => {
                "local dashboard budget policy unavailable"
            }
            DashboardServerErrorCode::ResourceLimit => "local dashboard resource limit exceeded",
            DashboardServerErrorCode::Protocol => "local dashboard request rejected",
            DashboardServerErrorCode::InternalFailure => "local dashboard operation failed",
        })
    }
}

impl Error for DashboardServerError {}

/// Closed loopback address family selectable by the local CLI.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LoopbackFamily {
    /// Bind `127.0.0.1` only.
    Ipv4,
    /// Bind `::1` only.
    Ipv6,
}

/// Bounded startup configuration for one foreground process.
#[derive(Clone, Debug)]
pub struct DashboardServerConfig {
    /// Existing cache root containing the metadata-only audit database.
    pub audit_root: PathBuf,
    /// Existing exact-owned local budget-policy root.
    pub policy_root: PathBuf,
    /// Closed loopback family.
    pub family: LoopbackFamily,
    /// Maximum recent audit rows in one recovery snapshot.
    pub max_records: u64,
    /// Minimum interval between audit-store reads.
    pub poll_interval: Duration,
    /// Maximum retained stream frames.
    pub max_stream_frames: usize,
    /// Maximum retained serialized stream bytes.
    pub max_stream_bytes: usize,
    /// Maximum stream-frame age.
    pub max_stream_age: Duration,
    /// Per-request read/write timeout.
    pub request_timeout: Duration,
}

impl DashboardServerConfig {
    /// Creates the frozen DBC-3 local resource profile.
    #[must_use]
    pub fn local(audit_root: PathBuf, policy_root: PathBuf) -> Self {
        Self {
            audit_root,
            policy_root,
            family: LoopbackFamily::Ipv4,
            max_records: 256,
            poll_interval: Duration::from_millis(250),
            max_stream_frames: 64,
            max_stream_bytes: 524_288,
            max_stream_age: Duration::from_mins(5),
            request_timeout: Duration::from_secs(2),
        }
    }

    fn validate(&self) -> Result<(), DashboardServerError> {
        if self.max_records == 0
            || self.max_records > 1_000
            || !(Duration::from_millis(100)..=Duration::from_secs(5)).contains(&self.poll_interval)
            || self.max_stream_frames == 0
            || self.max_stream_frames > 256
            || !(65_536..=2_097_152).contains(&self.max_stream_bytes)
            || !(Duration::from_secs(10)..=Duration::from_mins(10)).contains(&self.max_stream_age)
            || !(Duration::from_millis(250)..=Duration::from_secs(10))
                .contains(&self.request_timeout)
        {
            return Err(DashboardServerError::new(
                DashboardServerErrorCode::InvalidConfiguration,
            ));
        }
        Ok(())
    }
}

/// Source-free readiness record printed exactly once before the foreground loop.
#[derive(Clone, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct DashboardReady {
    /// Schema discriminator.
    pub schema_name: String,
    /// Contract version.
    pub schema_version: String,
    /// Exact loopback origin.
    pub origin: String,
    /// Fragment-bearing one-use bootstrap URL.
    pub bootstrap_url: String,
    /// Digest of the three compiled immutable assets.
    pub asset_sha256: String,
}

/// Explicit process-local shutdown capability for embedding and tests.
#[derive(Clone)]
pub struct DashboardShutdown {
    running: Arc<AtomicBool>,
}

impl DashboardShutdown {
    /// Requests termination of the accept loop, streams, and polling.
    pub fn request(&self) {
        self.running.store(false, Ordering::SeqCst);
    }

    /// Reports whether this foreground session is still active.
    #[must_use]
    pub fn is_running(&self) -> bool {
        self.running.load(Ordering::SeqCst)
    }
}

struct StreamFrame {
    sequence: u64,
    payload: Arc<Vec<u8>>,
    created_at: Instant,
}

struct Coordinator {
    bootstrap: Option<String>,
    api_base: String,
    snapshot: DashboardSnapshot,
    snapshot_bytes: Arc<Vec<u8>>,
    frames: VecDeque<StreamFrame>,
    frame_bytes: usize,
}

struct Shared {
    running: Arc<AtomicBool>,
    active_connections: AtomicUsize,
    active_streams: AtomicUsize,
    coordinator: Mutex<Coordinator>,
}

/// One bound foreground server. It performs no work until [`Self::run`] is called.
pub struct DashboardServer {
    listener: TcpListener,
    reader: AuditReader,
    shared: Arc<Shared>,
    config: DashboardServerConfig,
    host: String,
    origin: String,
}

impl DashboardServer {
    /// Validates explicit roots, binds one verified loopback socket, and creates
    /// independent process-local bootstrap and API-route capabilities.
    ///
    /// # Errors
    ///
    /// Fails closed for unsafe roots, overlap, invalid stores, randomness
    /// failure, non-loopback resolution, or resource-profile violations.
    pub fn bind(
        config: DashboardServerConfig,
    ) -> Result<(Self, DashboardReady), DashboardServerError> {
        config.validate()?;
        let reader = AuditReader::open(&config.audit_root)
            .map_err(|_| DashboardServerError::new(DashboardServerErrorCode::AuditUnavailable))?;
        PolicyStore::open(&config.policy_root)
            .map_err(|_| DashboardServerError::new(DashboardServerErrorCode::PolicyUnavailable))?;
        validate_distinct_roots(&config.audit_root, &config.policy_root)?;
        let requested = match config.family {
            LoopbackFamily::Ipv4 => SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 0),
            LoopbackFamily::Ipv6 => SocketAddr::new(IpAddr::V6(Ipv6Addr::LOCALHOST), 0),
        };
        let listener = TcpListener::bind(requested)
            .map_err(|_| DashboardServerError::new(DashboardServerErrorCode::BindFailure))?;
        listener
            .set_nonblocking(true)
            .map_err(|_| DashboardServerError::new(DashboardServerErrorCode::BindFailure))?;
        let address = listener
            .local_addr()
            .map_err(|_| DashboardServerError::new(DashboardServerErrorCode::BindFailure))?;
        if !address.ip().is_loopback() || address.port() == 0 {
            return Err(DashboardServerError::new(
                DashboardServerErrorCode::BindFailure,
            ));
        }
        let host = host_for(address);
        let origin = format!("http://{host}");
        let bootstrap = random_hex()?;
        let route = random_hex()?;
        let api_base = format!("/api/session/{route}");
        let (snapshot, snapshot_bytes) = read_snapshot(&reader, config.max_records, 1)?;
        let running = Arc::new(AtomicBool::new(true));
        let shared = Arc::new(Shared {
            running: Arc::clone(&running),
            active_connections: AtomicUsize::new(0),
            active_streams: AtomicUsize::new(0),
            coordinator: Mutex::new(Coordinator {
                bootstrap: Some(bootstrap.clone()),
                api_base,
                snapshot,
                snapshot_bytes: Arc::new(snapshot_bytes.clone()),
                frames: VecDeque::from([StreamFrame {
                    sequence: 1,
                    payload: Arc::new(snapshot_bytes),
                    created_at: Instant::now(),
                }]),
                frame_bytes: 0,
            }),
        });
        {
            let mut coordinator = shared.coordinator.lock().map_err(|_| {
                DashboardServerError::new(DashboardServerErrorCode::InternalFailure)
            })?;
            coordinator.frame_bytes = coordinator.snapshot_bytes.len();
        }
        let ready = DashboardReady {
            schema_name: "dashboard-ready".into(),
            schema_version: VERSION.into(),
            origin: origin.clone(),
            bootstrap_url: format!("{origin}/#{bootstrap}"),
            asset_sha256: asset_identity(),
        };
        Ok((
            Self {
                listener,
                reader,
                shared,
                config,
                host,
                origin,
            },
            ready,
        ))
    }

    /// Returns an explicit local handle that cannot grant HTTP access.
    #[must_use]
    pub fn shutdown_handle(&self) -> DashboardShutdown {
        DashboardShutdown {
            running: Arc::clone(&self.shared.running),
        }
    }

    /// Runs bounded audit polling and local request handling until shutdown.
    ///
    /// # Errors
    ///
    /// Fails closed when the audit store becomes unavailable or the accept loop
    /// encounters a non-transient operating-system failure.
    pub fn run(self) -> Result<(), DashboardServerError> {
        let mut workers = Vec::new();
        let mut last_poll = Instant::now()
            .checked_sub(self.config.poll_interval)
            .unwrap_or_else(Instant::now);
        let mut failure = None;
        while self.shared.running.load(Ordering::SeqCst) {
            if last_poll.elapsed() >= self.config.poll_interval {
                if let Err(error) = refresh_snapshot(&self.reader, &self.shared, &self.config) {
                    self.shared.running.store(false, Ordering::SeqCst);
                    failure = Some(error);
                    break;
                }
                last_poll = Instant::now();
            }
            reap_finished(&mut workers);
            match self.listener.accept() {
                Ok((stream, address)) => {
                    if !address.ip().is_loopback() {
                        reject_busy(stream);
                        continue;
                    }
                    if self
                        .shared
                        .active_connections
                        .fetch_add(1, Ordering::SeqCst)
                        >= MAX_CONNECTIONS
                    {
                        self.shared
                            .active_connections
                            .fetch_sub(1, Ordering::SeqCst);
                        reject_busy(stream);
                        continue;
                    }
                    let shared = Arc::clone(&self.shared);
                    let host = self.host.clone();
                    let origin = self.origin.clone();
                    let policy_root = self.config.policy_root.clone();
                    let timeout = self.config.request_timeout;
                    let stream_poll = self.config.poll_interval;
                    workers.push(thread::spawn(move || {
                        let _guard = ConnectionGuard {
                            shared: Arc::clone(&shared),
                        };
                        let _ = handle_connection(
                            stream,
                            &shared,
                            &host,
                            &origin,
                            &policy_root,
                            timeout,
                            stream_poll,
                        );
                    }));
                }
                Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => {
                    thread::sleep(Duration::from_millis(10));
                }
                Err(_) => {
                    self.shared.running.store(false, Ordering::SeqCst);
                    failure = Some(DashboardServerError::new(
                        DashboardServerErrorCode::InternalFailure,
                    ));
                    break;
                }
            }
        }
        self.shared.running.store(false, Ordering::SeqCst);
        if workers.into_iter().any(|worker| worker.join().is_err()) {
            return Err(DashboardServerError::new(
                DashboardServerErrorCode::InternalFailure,
            ));
        }
        failure.map_or(Ok(()), Err)
    }
}

struct ConnectionGuard {
    shared: Arc<Shared>,
}

impl Drop for ConnectionGuard {
    fn drop(&mut self) {
        self.shared
            .active_connections
            .fetch_sub(1, Ordering::SeqCst);
    }
}

struct StreamGuard<'a> {
    shared: &'a Shared,
}

impl Drop for StreamGuard<'_> {
    fn drop(&mut self) {
        self.shared.active_streams.fetch_sub(1, Ordering::SeqCst);
    }
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ApplyRequest {
    draft: LocalBudgetPolicyDraft,
    expected_policy_id: Option<String>,
    expected_revision: Option<String>,
    expected_preview_receipt_id: Option<String>,
    apply: bool,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RemoveRequest {
    expected_policy_id: String,
    expected_revision: String,
    expected_preview_receipt_id: Option<String>,
    apply: bool,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RollbackRequest {
    expected_policy_id: Option<String>,
    expected_revision: Option<String>,
    expected_preview_receipt_id: Option<String>,
    apply: bool,
}

#[derive(Serialize)]
struct StateResponse {
    snapshot: DashboardSnapshot,
    policy: PolicyStoreState,
}

#[derive(Serialize)]
struct BootstrapResponse {
    schema_name: &'static str,
    schema_version: &'static str,
    api_base: String,
}

fn handle_connection(
    mut stream: TcpStream,
    shared: &Shared,
    host: &str,
    origin: &str,
    policy_root: &Path,
    timeout: Duration,
    stream_poll: Duration,
) -> Result<(), DashboardServerError> {
    stream
        .set_read_timeout(Some(timeout))
        .and_then(|()| stream.set_write_timeout(Some(timeout)))
        .map_err(|_| DashboardServerError::new(DashboardServerErrorCode::Protocol))?;
    let request = match http::read_request(&mut stream) {
        Ok(request) => request,
        Err(error) => {
            return write_error(&mut stream, status_for(error.code()), error.code());
        }
    };
    if request.header("host") != Some(host) {
        return write_error(&mut stream, 421, DashboardServerErrorCode::Protocol);
    }
    if let Some(supplied_origin) = request.header("origin")
        && supplied_origin != origin
    {
        return write_error(&mut stream, 403, DashboardServerErrorCode::Protocol);
    }
    let api_target = shared
        .coordinator
        .lock()
        .map_err(|_| internal_error())?
        .api_base
        .clone();
    let api_target = request
        .target
        .strip_prefix(&api_target)
        .filter(|target| target.starts_with('/'))
        .map(str::to_owned);
    match (request.method.as_str(), request.target.as_str()) {
        ("GET", "/") => write_asset(&mut stream, "text/html; charset=utf-8", INDEX_HTML),
        ("GET", "/app.js") => write_asset(&mut stream, "text/javascript; charset=utf-8", APP_JS),
        ("GET", "/app.css") => write_asset(&mut stream, "text/css; charset=utf-8", APP_CSS),
        ("POST", "/api/bootstrap") => bootstrap(&mut stream, &request, shared, origin),
        _ if api_target.is_some() => {
            let api_target = api_target.as_deref().ok_or_else(internal_error)?;
            let state_changing = request.method != "GET";
            if state_changing && request.header("origin") != Some(origin) {
                return write_error(&mut stream, 403, DashboardServerErrorCode::Protocol);
            }
            if request.method == "GET" && api_target == "/events" {
                let Ok(cursor) = request
                    .header("last-event-id")
                    .map(str::parse::<u64>)
                    .transpose()
                else {
                    return write_error(&mut stream, 400, DashboardServerErrorCode::Protocol);
                };
                return stream_events(&mut stream, cursor, shared, stream_poll);
            }
            match route_api(&mut stream, &request, api_target, shared, policy_root) {
                Ok(()) => Ok(()),
                Err(error) => write_error(&mut stream, status_for(error.code()), error.code()),
            }
        }
        _ => write_error(&mut stream, 404, DashboardServerErrorCode::Protocol),
    }
}

fn route_api(
    stream: &mut TcpStream,
    request: &Request,
    target: &str,
    shared: &Shared,
    policy_root: &Path,
) -> Result<(), DashboardServerError> {
    match (request.method.as_str(), target) {
        ("GET", "/state") => {
            let snapshot = shared
                .coordinator
                .lock()
                .map_err(|_| internal_error())?
                .snapshot
                .clone();
            let policy = PolicyStore::open(policy_root)
                .and_then(|store| store.state())
                .map_err(policy_error)?;
            write_json(stream, 200, &StateResponse { snapshot, policy })
        }
        ("POST", "/policy/apply") => apply_policy(stream, request, policy_root),
        ("POST", "/policy/remove") => remove_policy(stream, request, policy_root),
        ("POST", "/policy/rollback") => rollback_policy(stream, request, policy_root),
        ("POST", "/shutdown") => {
            require_empty_write(request)?;
            write_json(
                stream,
                200,
                &serde_json::json!({"schema_name":"dashboard-shutdown","schema_version":VERSION,"stopped":true}),
            )?;
            shared.running.store(false, Ordering::SeqCst);
            Ok(())
        }
        ("GET" | "POST", _) => write_error(stream, 404, DashboardServerErrorCode::Protocol),
        _ => write_error(stream, 405, DashboardServerErrorCode::Protocol),
    }
}

fn apply_policy(
    stream: &mut TcpStream,
    request: &Request,
    policy_root: &Path,
) -> Result<(), DashboardServerError> {
    require_json_write(request)?;
    let input: ApplyRequest = parse_json(&request.body)?;
    let policy = compile_policy(input.draft).map_err(policy_error)?;
    let preview = PolicyStore::preview_apply(
        policy_root,
        &policy,
        input.expected_policy_id.as_deref(),
        input.expected_revision.as_deref(),
    )
    .map_err(policy_error)?;
    require_exact_preview(
        &preview.receipt_id,
        input.expected_preview_receipt_id.as_deref(),
        input.apply,
    )?;
    let receipt = if input.apply {
        PolicyStore::apply(
            policy_root,
            policy,
            input.expected_policy_id.as_deref(),
            input.expected_revision.as_deref(),
        )
        .map_err(policy_error)?
    } else {
        preview
    };
    write_json(stream, 200, &receipt)
}

fn remove_policy(
    stream: &mut TcpStream,
    request: &Request,
    policy_root: &Path,
) -> Result<(), DashboardServerError> {
    require_json_write(request)?;
    let input: RemoveRequest = parse_json(&request.body)?;
    let preview = PolicyStore::preview_remove(
        policy_root,
        &input.expected_policy_id,
        &input.expected_revision,
    )
    .map_err(policy_error)?;
    require_exact_preview(
        &preview.receipt_id,
        input.expected_preview_receipt_id.as_deref(),
        input.apply,
    )?;
    let receipt = if input.apply {
        PolicyStore::remove(
            policy_root,
            &input.expected_policy_id,
            &input.expected_revision,
        )
        .map_err(policy_error)?
    } else {
        preview
    };
    write_json(stream, 200, &receipt)
}

fn rollback_policy(
    stream: &mut TcpStream,
    request: &Request,
    policy_root: &Path,
) -> Result<(), DashboardServerError> {
    require_json_write(request)?;
    let input: RollbackRequest = parse_json(&request.body)?;
    let preview = PolicyStore::preview_rollback(
        policy_root,
        input.expected_policy_id.as_deref(),
        input.expected_revision.as_deref(),
    )
    .map_err(policy_error)?;
    require_exact_preview(
        &preview.receipt_id,
        input.expected_preview_receipt_id.as_deref(),
        input.apply,
    )?;
    let receipt = if input.apply {
        PolicyStore::rollback(
            policy_root,
            input.expected_policy_id.as_deref(),
            input.expected_revision.as_deref(),
        )
        .map_err(policy_error)?
    } else {
        preview
    };
    write_json(stream, 200, &receipt)
}

fn require_exact_preview(
    actual: &str,
    expected: Option<&str>,
    apply: bool,
) -> Result<(), DashboardServerError> {
    let valid = if apply {
        expected.is_some_and(|value| constant_time_equal(actual.as_bytes(), value.as_bytes()))
    } else {
        expected.is_none()
    };
    if !valid {
        return Err(DashboardServerError::new(
            DashboardServerErrorCode::PolicyUnavailable,
        ));
    }
    Ok(())
}

fn bootstrap(
    stream: &mut TcpStream,
    request: &Request,
    shared: &Shared,
    origin: &str,
) -> Result<(), DashboardServerError> {
    if request.header("origin") != Some(origin)
        || request.header("x-impresari-csrf") != Some("bootstrap")
        || !request.body.is_empty()
    {
        return write_error(stream, 403, DashboardServerErrorCode::Protocol);
    }
    let supplied = request.header("x-impresari-bootstrap").unwrap_or_default();
    let mut coordinator = shared.coordinator.lock().map_err(|_| internal_error())?;
    let accepted = coordinator
        .bootstrap
        .as_deref()
        .is_some_and(|expected| constant_time_equal(expected.as_bytes(), supplied.as_bytes()));
    if !accepted {
        return write_error(stream, 403, DashboardServerErrorCode::Protocol);
    }
    coordinator.bootstrap = None;
    let api_base = coordinator.api_base.clone();
    drop(coordinator);
    write_json(
        stream,
        200,
        &BootstrapResponse {
            schema_name: "dashboard-bootstrap",
            schema_version: VERSION,
            api_base,
        },
    )
}

fn stream_events(
    stream: &mut TcpStream,
    mut cursor: Option<u64>,
    shared: &Shared,
    poll: Duration,
) -> Result<(), DashboardServerError> {
    if shared.active_streams.fetch_add(1, Ordering::SeqCst) >= MAX_STREAMS {
        shared.active_streams.fetch_sub(1, Ordering::SeqCst);
        return write_error(stream, 429, DashboardServerErrorCode::ResourceLimit);
    }
    let _guard = StreamGuard { shared };
    http::write_sse_header(stream)?;
    let mut last_heartbeat = Instant::now();
    while shared.running.load(Ordering::SeqCst) {
        let (event, next) = next_stream_event(shared, cursor)?;
        if let Some(bytes) = event {
            stream
                .write_all(&bytes)
                .map_err(|_| DashboardServerError::new(DashboardServerErrorCode::Protocol))?;
            stream
                .flush()
                .map_err(|_| DashboardServerError::new(DashboardServerErrorCode::Protocol))?;
            cursor = Some(next);
        } else if last_heartbeat.elapsed() >= Duration::from_secs(15) {
            stream
                .write_all(b": keepalive\n\n")
                .map_err(|_| DashboardServerError::new(DashboardServerErrorCode::Protocol))?;
            last_heartbeat = Instant::now();
        }
        thread::sleep(poll);
    }
    Ok(())
}

fn next_stream_event(
    shared: &Shared,
    cursor: Option<u64>,
) -> Result<(Option<Vec<u8>>, u64), DashboardServerError> {
    let coordinator = shared.coordinator.lock().map_err(|_| internal_error())?;
    let current = coordinator
        .snapshot
        .stream_sequence
        .parse::<u64>()
        .map_err(|_| internal_error())?;
    let Some(cursor) = cursor else {
        return Ok((
            Some(sse_bytes("snapshot", current, &coordinator.snapshot_bytes)),
            current,
        ));
    };
    let earliest = coordinator
        .frames
        .front()
        .map_or(current, |frame| frame.sequence);
    if cursor > current || cursor.saturating_add(1) < earliest {
        return Ok((
            Some(sse_bytes(
                "reset_required",
                current,
                &coordinator.snapshot_bytes,
            )),
            current,
        ));
    }
    if let Some(frame) = coordinator
        .frames
        .iter()
        .find(|frame| frame.sequence > cursor)
    {
        return Ok((
            Some(sse_bytes("snapshot", frame.sequence, &frame.payload)),
            frame.sequence,
        ));
    }
    Ok((None, cursor))
}

fn sse_bytes(event: &str, sequence: u64, payload: &[u8]) -> Vec<u8> {
    let mut bytes = format!("event: {event}\nid: {sequence}\ndata: ").into_bytes();
    bytes.extend_from_slice(payload);
    bytes.extend_from_slice(b"\n\n");
    bytes
}

fn refresh_snapshot(
    reader: &AuditReader,
    shared: &Shared,
    config: &DashboardServerConfig,
) -> Result<(), DashboardServerError> {
    let current_sequence = shared
        .coordinator
        .lock()
        .map_err(|_| internal_error())?
        .snapshot
        .stream_sequence
        .parse::<u64>()
        .map_err(|_| internal_error())?;
    let next_sequence = current_sequence
        .checked_add(1)
        .ok_or_else(|| DashboardServerError::new(DashboardServerErrorCode::ResourceLimit))?;
    let (candidate, bytes) = read_snapshot(reader, config.max_records, next_sequence)?;
    let mut coordinator = shared.coordinator.lock().map_err(|_| internal_error())?;
    if candidate.records == coordinator.snapshot.records
        && candidate.aggregates == coordinator.snapshot.aggregates
        && candidate.unavailable_rows == coordinator.snapshot.unavailable_rows
    {
        return Ok(());
    }
    let payload = Arc::new(bytes);
    coordinator.frame_bytes = coordinator
        .frame_bytes
        .checked_add(payload.len())
        .ok_or_else(|| DashboardServerError::new(DashboardServerErrorCode::ResourceLimit))?;
    coordinator.frames.push_back(StreamFrame {
        sequence: next_sequence,
        payload: Arc::clone(&payload),
        created_at: Instant::now(),
    });
    coordinator.snapshot = candidate;
    coordinator.snapshot_bytes = payload;
    while coordinator.frames.len() > config.max_stream_frames
        || coordinator.frame_bytes > config.max_stream_bytes
        || coordinator
            .frames
            .front()
            .is_some_and(|frame| frame.created_at.elapsed() > config.max_stream_age)
    {
        if let Some(removed) = coordinator.frames.pop_front() {
            coordinator.frame_bytes = coordinator
                .frame_bytes
                .saturating_sub(removed.payload.len());
        } else {
            break;
        }
    }
    Ok(())
}

fn read_snapshot(
    reader: &AuditReader,
    maximum: u64,
    sequence: u64,
) -> Result<(DashboardSnapshot, Vec<u8>), DashboardServerError> {
    let batch = reader
        .recent(maximum)
        .map_err(|_| DashboardServerError::new(DashboardServerErrorCode::AuditUnavailable))?;
    let mut unavailable = batch.unavailable_rows;
    let mut records = Vec::new();
    for event in batch.events {
        match project_event(&event) {
            Ok(record) => records.push(record),
            Err(_) => {
                unavailable = unavailable.checked_add(1).ok_or_else(|| {
                    DashboardServerError::new(DashboardServerErrorCode::ResourceLimit)
                })?;
            }
        }
    }
    bounded_snapshot(sequence, records, unavailable)
}

fn bounded_snapshot(
    sequence: u64,
    mut records: Vec<DashboardRecord>,
    mut unavailable: u64,
) -> Result<(DashboardSnapshot, Vec<u8>), DashboardServerError> {
    loop {
        let snapshot = build_snapshot(sequence, records.clone(), unavailable)
            .map_err(|_| DashboardServerError::new(DashboardServerErrorCode::ResourceLimit))?;
        let bytes = serde_json::to_vec(&snapshot).map_err(|_| internal_error())?;
        if bytes.len() <= MAX_SNAPSHOT_BYTES {
            return Ok((snapshot, bytes));
        }
        if records.pop().is_none() {
            return Err(DashboardServerError::new(
                DashboardServerErrorCode::ResourceLimit,
            ));
        }
        unavailable = unavailable
            .checked_add(1)
            .ok_or_else(|| DashboardServerError::new(DashboardServerErrorCode::ResourceLimit))?;
    }
}

fn validate_distinct_roots(audit: &Path, policy: &Path) -> Result<(), DashboardServerError> {
    let audit = audit
        .canonicalize()
        .map_err(|_| DashboardServerError::new(DashboardServerErrorCode::AuditUnavailable))?;
    let policy = policy
        .canonicalize()
        .map_err(|_| DashboardServerError::new(DashboardServerErrorCode::PolicyUnavailable))?;
    if audit == policy || audit.starts_with(&policy) || policy.starts_with(&audit) {
        return Err(DashboardServerError::new(
            DashboardServerErrorCode::InvalidConfiguration,
        ));
    }
    Ok(())
}

fn random_hex() -> Result<String, DashboardServerError> {
    let mut bytes = [0_u8; 32];
    getrandom::fill(&mut bytes).map_err(|_| internal_error())?;
    let mut value = String::with_capacity(64);
    for byte in bytes {
        use fmt::Write as _;
        write!(value, "{byte:02x}").map_err(|_| internal_error())?;
    }
    Ok(value)
}

fn host_for(address: SocketAddr) -> String {
    match address {
        SocketAddr::V4(value) => value.to_string(),
        SocketAddr::V6(value) => format!("[{}]:{}", value.ip(), value.port()),
    }
}

fn asset_identity() -> String {
    let mut hasher = Sha256::new();
    hasher.update(b"impresari-context\0dashboard-assets\0");
    for asset in [INDEX_HTML, APP_JS, APP_CSS] {
        hasher.update((asset.len() as u64).to_be_bytes());
        hasher.update(asset);
    }
    let digest = hasher.finalize();
    let hex = digest
        .iter()
        .fold(String::with_capacity(64), |mut output, byte| {
            use fmt::Write as _;
            write!(output, "{byte:02x}").expect("writing to a string cannot fail");
            output
        });
    format!("sha256:{hex}")
}

fn constant_time_equal(left: &[u8], right: &[u8]) -> bool {
    if left.len() != right.len() {
        return false;
    }
    left.iter()
        .zip(right)
        .fold(0_u8, |difference, (left, right)| {
            difference | (left ^ right)
        })
        == 0
}

fn require_json_write(request: &Request) -> Result<(), DashboardServerError> {
    if request.header("x-impresari-csrf") != Some("1")
        || request.header("content-type") != Some("application/json")
        || request.body.is_empty()
    {
        return Err(DashboardServerError::new(
            DashboardServerErrorCode::Protocol,
        ));
    }
    Ok(())
}

fn require_empty_write(request: &Request) -> Result<(), DashboardServerError> {
    if request.header("x-impresari-csrf") != Some("1") || !request.body.is_empty() {
        return Err(DashboardServerError::new(
            DashboardServerErrorCode::Protocol,
        ));
    }
    Ok(())
}

fn parse_json<T: for<'de> Deserialize<'de>>(bytes: &[u8]) -> Result<T, DashboardServerError> {
    serde_json::from_slice(bytes)
        .map_err(|_| DashboardServerError::new(DashboardServerErrorCode::InvalidConfiguration))
}

fn policy_error(error: DashboardError) -> DashboardServerError {
    DashboardServerError::new(match error.code() {
        DashboardErrorCode::ResourceLimit => DashboardServerErrorCode::ResourceLimit,
        DashboardErrorCode::StaleState
        | DashboardErrorCode::InvalidInput
        | DashboardErrorCode::IntegrityFailure
        | DashboardErrorCode::IncompatibleData
        | DashboardErrorCode::StorageFailure => DashboardServerErrorCode::PolicyUnavailable,
    })
}

fn internal_error() -> DashboardServerError {
    DashboardServerError::new(DashboardServerErrorCode::InternalFailure)
}

fn status_for(code: DashboardServerErrorCode) -> u16 {
    match code {
        DashboardServerErrorCode::InvalidConfiguration | DashboardServerErrorCode::Protocol => 400,
        DashboardServerErrorCode::ResourceLimit => 413,
        DashboardServerErrorCode::PolicyUnavailable => 409,
        DashboardServerErrorCode::AuditUnavailable
        | DashboardServerErrorCode::BindFailure
        | DashboardServerErrorCode::InternalFailure => 503,
    }
}

fn error_name(code: DashboardServerErrorCode) -> &'static str {
    match code {
        DashboardServerErrorCode::InvalidConfiguration => "invalid_configuration",
        DashboardServerErrorCode::BindFailure => "bind_failure",
        DashboardServerErrorCode::AuditUnavailable => "audit_unavailable",
        DashboardServerErrorCode::PolicyUnavailable => "policy_unavailable",
        DashboardServerErrorCode::ResourceLimit => "resource_limit",
        DashboardServerErrorCode::Protocol => "request_rejected",
        DashboardServerErrorCode::InternalFailure => "internal_failure",
    }
}

fn write_error(
    stream: &mut TcpStream,
    status: u16,
    code: DashboardServerErrorCode,
) -> Result<(), DashboardServerError> {
    let body = serde_json::to_vec(&serde_json::json!({
        "schema_name": "dashboard-http-error",
        "schema_version": VERSION,
        "code": error_name(code)
    }))
    .map_err(|_| internal_error())?;
    http::write_response(
        stream,
        Response {
            status,
            content_type: "application/json",
            body: &body,
        },
    )
}

fn write_json(
    stream: &mut TcpStream,
    status: u16,
    value: &impl Serialize,
) -> Result<(), DashboardServerError> {
    let body = serde_json::to_vec(value).map_err(|_| internal_error())?;
    if body.len() > MAX_SNAPSHOT_BYTES {
        return Err(DashboardServerError::new(
            DashboardServerErrorCode::ResourceLimit,
        ));
    }
    http::write_response(
        stream,
        Response {
            status,
            content_type: "application/json",
            body: &body,
        },
    )
}

fn write_asset(
    stream: &mut TcpStream,
    content_type: &'static str,
    body: &[u8],
) -> Result<(), DashboardServerError> {
    http::write_response(
        stream,
        Response {
            status: 200,
            content_type,
            body,
        },
    )
}

fn reject_busy(mut stream: TcpStream) {
    let _ = write_error(&mut stream, 429, DashboardServerErrorCode::ResourceLimit);
}

fn reap_finished(workers: &mut Vec<thread::JoinHandle<()>>) {
    let mut index = 0;
    while index < workers.len() {
        if workers[index].is_finished() {
            let worker = workers.swap_remove(index);
            let _ = worker.join();
        } else {
            index += 1;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use context_core::{AuditOutcome, Capability, ResourceBudget, audit_event};
    use context_dashboard::{BudgetCeilings, BudgetSelector, LocalBudgetRule};
    use context_store::{AuditRetention, AuditStore};
    use rusqlite::{Connection, params};
    use std::{fs, io::Read as _, sync::atomic::AtomicU64};

    static TEST_SEQUENCE: AtomicU64 = AtomicU64::new(0);

    struct TestRoot(PathBuf);

    impl TestRoot {
        fn new(label: &str) -> Self {
            let sequence = TEST_SEQUENCE.fetch_add(1, Ordering::Relaxed);
            let path = std::env::temp_dir().join(format!(
                "impresari-dashboard-server-{label}-{}-{sequence}",
                std::process::id()
            ));
            fs::create_dir(&path).expect("root");
            Self(path)
        }
    }

    impl Drop for TestRoot {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }

    fn budget() -> ResourceBudget {
        ResourceBudget::conservative(8_192, 8, 16, 512, 32, 4, 5_000, 8_388_608).expect("budget")
    }

    fn policy(revision: &str, requested: &str) -> context_dashboard::LocalBudgetPolicy {
        compile_policy(LocalBudgetPolicyDraft {
            schema_name: "local-budget-policy".into(),
            schema_version: VERSION.into(),
            revision: revision.into(),
            created_at: "2026-08-30T00:00:00Z".into(),
            expires_at: None,
            rules: vec![LocalBudgetRule {
                rule_id: "local".into(),
                selector: BudgetSelector {
                    purpose: None,
                    capability: Some(Capability::ContextBuild),
                },
                deny: false,
                ceilings: BudgetCeilings {
                    requested: Some(requested.into()),
                    ..BudgetCeilings::default()
                },
            }],
        })
        .expect("policy")
    }

    fn fixture() -> (TestRoot, PathBuf, PathBuf) {
        let root = TestRoot::new("fixture");
        let audit_root = root.0.join("audit-cache");
        let policy_root = root.0.join("policy-state");
        let mut audit = AuditStore::open(&audit_root).expect("audit");
        let identity = format!("sha256:{}", "a".repeat(64));
        let event = audit_event(
            "evt_dashboard_server01",
            "req_dashboard_server01",
            "2026-08-30T00:00:00Z",
            Some(&identity),
            Some(&identity),
            Capability::ContextBuild,
            AuditOutcome::Limited,
            &identity,
            budget(),
            7,
            "0.1.0",
        )
        .expect("event");
        audit
            .append(
                &event,
                &AuditRetention::new("2026-08-01T00:00:00Z", 100, 1_048_576).expect("retention"),
            )
            .expect("append");
        drop(audit);
        let connection = Connection::open(audit_root.join("audit/audit.sqlite3"))
            .expect("open synthetic rejected row");
        connection
            .execute(
                "INSERT INTO audit_events(event_id,occurred_at,workspace_identity,payload) VALUES(?1,?2,NULL,?3)",
                params![
                    "evt_dashboard_server_rejected",
                    "2026-08-30T00:00:01Z",
                    br#"{"schema_name":"future-audit-event","source":"DBC4_UNIT_SOURCE_CANARY"}"#
                ],
            )
            .expect("insert synthetic rejected row");
        drop(connection);
        let initial = policy("1", "4096");
        PolicyStore::apply(&policy_root, initial, None, None).expect("policy state");
        (root, audit_root, policy_root)
    }

    fn exchange(origin: &str, request: &str) -> Vec<u8> {
        let address = origin.strip_prefix("http://").expect("origin");
        let mut stream = TcpStream::connect(address).expect("connect");
        stream.write_all(request.as_bytes()).expect("write");
        let mut response = Vec::new();
        stream.read_to_end(&mut response).expect("read");
        response
    }

    fn exchange_prefix(origin: &str, request: &str, needle: &[u8]) -> Vec<u8> {
        let address = origin.strip_prefix("http://").expect("origin");
        let mut stream = TcpStream::connect(address).expect("connect");
        stream
            .set_read_timeout(Some(Duration::from_secs(2)))
            .expect("timeout");
        stream.write_all(request.as_bytes()).expect("write");
        let mut response = Vec::new();
        let mut chunk = [0_u8; 4_096];
        while !response.windows(needle.len()).any(|part| part == needle) {
            match stream.read(&mut chunk) {
                Ok(0) | Err(_) => break,
                Ok(count) => response.extend_from_slice(&chunk[..count]),
            }
        }
        response
    }

    #[test]
    #[allow(clippy::too_many_lines)]
    fn foreground_server_is_loopback_bootstrap_bound_and_exactly_stoppable() {
        let (_root, audit_root, policy_root) = fixture();
        let (server, ready) = DashboardServer::bind(DashboardServerConfig::local(
            audit_root,
            policy_root.clone(),
        ))
        .expect("bind");
        let shutdown = server.shutdown_handle();
        let origin = ready.origin.clone();
        let host = origin.strip_prefix("http://").expect("host").to_owned();
        let token = ready
            .bootstrap_url
            .split_once('#')
            .expect("fragment")
            .1
            .to_owned();
        let worker = thread::spawn(move || server.run());

        let wrong_host = exchange(&origin, "GET / HTTP/1.1\r\nHost: attacker.invalid\r\n\r\n");
        assert!(wrong_host.starts_with(b"HTTP/1.1 421"));
        let page = exchange(&origin, &format!("GET / HTTP/1.1\r\nHost: {host}\r\n\r\n"));
        let page_text = String::from_utf8(page).expect("page");
        assert!(page_text.contains("Content-Security-Policy:"));
        assert!(page_text.contains("Local metadata only"));
        assert!(!page_text.contains(&token));

        let bootstrap = exchange(
            &origin,
            &format!(
                "POST /api/bootstrap HTTP/1.1\r\nHost: {host}\r\nOrigin: {origin}\r\nX-Impresari-CSRF: bootstrap\r\nX-Impresari-Bootstrap: {token}\r\nContent-Length: 0\r\n\r\n"
            ),
        );
        assert!(bootstrap.starts_with(b"HTTP/1.1 200"));
        let bootstrap_text = String::from_utf8(bootstrap.clone()).expect("bootstrap response");
        let bootstrap_value: serde_json::Value = serde_json::from_str(
            bootstrap_text
                .split_once("\r\n\r\n")
                .expect("bootstrap body")
                .1,
        )
        .expect("bootstrap json");
        let api_base = bootstrap_value["api_base"]
            .as_str()
            .expect("api base")
            .to_owned();
        assert!(api_base.starts_with("/api/session/"));
        assert!(!bootstrap_text.contains("Set-Cookie:"));
        let replay = exchange(
            &origin,
            &format!(
                "POST /api/bootstrap HTTP/1.1\r\nHost: {host}\r\nOrigin: {origin}\r\nX-Impresari-CSRF: bootstrap\r\nX-Impresari-Bootstrap: {token}\r\nContent-Length: 0\r\n\r\n"
            ),
        );
        assert!(replay.starts_with(b"HTTP/1.1 403"));

        let undisclosed_state = exchange(
            &origin,
            &format!("GET /api/state HTTP/1.1\r\nHost: {host}\r\n\r\n"),
        );
        assert!(undisclosed_state.starts_with(b"HTTP/1.1 404"));
        let hostile_origin_state = exchange(
            &origin,
            &format!(
                "GET {api_base}/state HTTP/1.1\r\nHost: {host}\r\nOrigin: http://attacker.invalid\r\n\r\n"
            ),
        );
        assert!(hostile_origin_state.starts_with(b"HTTP/1.1 403"));

        let state = exchange(
            &origin,
            &format!("GET {api_base}/state HTTP/1.1\r\nHost: {host}\r\n\r\n"),
        );
        let state_text = String::from_utf8(state).expect("state");
        assert!(state_text.starts_with("HTTP/1.1 200"));
        assert!(state_text.contains("dashboard-snapshot"));
        assert!(state_text.contains("\"records\":[{"));
        assert!(state_text.contains("\"unavailable_rows\":\"1\""));
        assert!(!state_text.contains("DBC4_UNIT_SOURCE_CANARY"));
        assert!(!state_text.contains(&token));

        let events = exchange_prefix(
            &origin,
            &format!("GET {api_base}/events HTTP/1.1\r\nHost: {host}\r\n\r\n"),
            b"event: snapshot",
        );
        assert!(events.starts_with(b"HTTP/1.1 200"));
        assert!(events.windows(15).any(|part| part == b"event: snapshot"));
        assert!(
            !events
                .windows(token.len())
                .any(|part| part == token.as_bytes())
        );
        let reset = exchange_prefix(
            &origin,
            &format!(
                "GET {api_base}/events HTTP/1.1\r\nHost: {host}\r\nLast-Event-ID: 999\r\n\r\n"
            ),
            b"event: reset_required",
        );
        assert!(
            reset
                .windows(21)
                .any(|part| part == b"event: reset_required")
        );

        let next = policy("2", "2048");
        let draft = LocalBudgetPolicyDraft {
            schema_name: next.schema_name,
            schema_version: next.schema_version,
            revision: next.revision,
            created_at: next.created_at,
            expires_at: next.expires_at,
            rules: next.rules,
        };
        let current = PolicyStore::open(&policy_root)
            .and_then(|store| store.state())
            .expect("current");
        let body = serde_json::to_string(&serde_json::json!({
            "draft": draft,
            "expected_policy_id": current.current_policy_id,
            "expected_revision": current.current_revision,
            "expected_preview_receipt_id": null,
            "apply": false
        }))
        .expect("body");
        let preview = exchange(
            &origin,
            &format!(
                "POST {api_base}/policy/apply HTTP/1.1\r\nHost: {host}\r\nOrigin: {origin}\r\nX-Impresari-CSRF: 1\r\nContent-Type: application/json\r\nContent-Length: {}\r\n\r\n{body}",
                body.len()
            ),
        );
        assert!(preview.starts_with(b"HTTP/1.1 200"));
        let preview_text = String::from_utf8(preview).expect("preview");
        assert!(preview_text.contains("\"external_write_performed\":false"));
        let preview_value: serde_json::Value = serde_json::from_str(
            preview_text
                .split_once("\r\n\r\n")
                .expect("response body")
                .1,
        )
        .expect("preview json");
        let receipt_id = preview_value["receipt_id"].as_str().expect("receipt");
        let mut apply_value: serde_json::Value = serde_json::from_str(&body).expect("apply body");
        apply_value["expected_preview_receipt_id"] = format!("sha256:{}", "f".repeat(64)).into();
        apply_value["apply"] = true.into();
        let wrong_apply_body = serde_json::to_string(&apply_value).expect("wrong apply body");
        let rejected_apply = exchange(
            &origin,
            &format!(
                "POST {api_base}/policy/apply HTTP/1.1\r\nHost: {host}\r\nOrigin: {origin}\r\nX-Impresari-CSRF: 1\r\nContent-Type: application/json\r\nContent-Length: {}\r\n\r\n{wrong_apply_body}",
                wrong_apply_body.len()
            ),
        );
        assert!(rejected_apply.starts_with(b"HTTP/1.1 409"));
        apply_value["expected_preview_receipt_id"] = receipt_id.into();
        let apply_body = serde_json::to_string(&apply_value).expect("apply body");
        let applied = exchange(
            &origin,
            &format!(
                "POST {api_base}/policy/apply HTTP/1.1\r\nHost: {host}\r\nOrigin: {origin}\r\nX-Impresari-CSRF: 1\r\nContent-Type: application/json\r\nContent-Length: {}\r\n\r\n{apply_body}",
                apply_body.len()
            ),
        );
        assert!(applied.starts_with(b"HTTP/1.1 200"));
        assert!(
            String::from_utf8(applied)
                .expect("applied")
                .contains("\"external_write_performed\":true")
        );
        assert_eq!(
            PolicyStore::open(&policy_root)
                .and_then(|store| store.state())
                .expect("applied state")
                .current_revision
                .as_deref(),
            Some("2")
        );

        let malformed = exchange(
            &origin,
            &format!(
                "POST {api_base}/policy/apply HTTP/1.1\r\nHost: {host}\r\nOrigin: {origin}\r\nX-Impresari-CSRF: 1\r\nContent-Type: application/json\r\nContent-Length: 1\r\n\r\n{{"
            ),
        );
        assert!(malformed.starts_with(b"HTTP/1.1 400"));

        let stopped = exchange(
            &origin,
            &format!(
                "POST {api_base}/shutdown HTTP/1.1\r\nHost: {host}\r\nOrigin: {origin}\r\nX-Impresari-CSRF: 1\r\nContent-Length: 0\r\n\r\n"
            ),
        );
        assert!(stopped.starts_with(b"HTTP/1.1 200"));
        assert!(worker.join().expect("join").is_ok());
        assert!(!shutdown.is_running());
        assert!(TcpStream::connect(host).is_err());
    }

    #[test]
    fn roots_must_be_existing_exact_and_disjoint() {
        let (_root, audit_root, policy_root) = fixture();
        let mut config = DashboardServerConfig::local(audit_root.clone(), policy_root);
        config.policy_root = audit_root;
        assert_eq!(
            DashboardServer::bind(config).err().expect("overlap").code(),
            DashboardServerErrorCode::PolicyUnavailable
        );
    }
}
