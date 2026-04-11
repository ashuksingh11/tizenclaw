use axum::{
    body::Body,
    extract::{Path, Query, Request, State},
    http::{header, HeaderValue, Response, StatusCode},
    response::{Html, IntoResponse, Json},
    routing::{get, post},
    Router,
};
use base64::{engine::general_purpose::STANDARD as BASE64, Engine as _};
use hyper::upgrade;
use hyper_util::rt::TokioIo;
use openssl::sha::sha1;
use serde::Deserialize;
use serde_json::{json, Value};
use std::{
    io::{ErrorKind, Read, Write},
    os::fd::FromRawFd,
    os::unix::net::UnixStream,
    path::{Path as StdPath, PathBuf},
    sync::atomic::{AtomicBool, AtomicU64, Ordering},
    time::{Duration, Instant},
};

static RUNNING: AtomicBool = AtomicBool::new(true);
static REQUEST_ID: AtomicU64 = AtomicU64::new(1);
const DEFAULT_DASHBOARD_PORT: u16 = 9090;
const DEFAULT_SOCKET_NAME: &str = "tizenclaw.sock";
const MAX_IPC_MESSAGE_SIZE: usize = 10 * 1024 * 1024;
const IPC_TIMEOUT: Duration = Duration::from_secs(120);

#[derive(Clone)]
struct AppState {
    web_root: PathBuf,
    socket_path: String,
    data_dir: PathBuf,
}

#[derive(Deserialize)]
struct PromptRequest {
    prompt: String,
    #[serde(default)]
    session_id: Option<String>,
}

#[derive(Deserialize)]
struct BackendConfigRequest {
    path: String,
    value: Value,
}

#[derive(Deserialize)]
struct AuditQuery {
    limit: Option<usize>,
}

#[derive(Deserialize)]
struct StreamRequest {
    prompt: String,
    #[serde(default)]
    session_id: Option<String>,
}

struct StderrLogger;

impl log::Log for StderrLogger {
    fn enabled(&self, _: &log::Metadata) -> bool {
        true
    }

    fn log(&self, record: &log::Record) {
        eprintln!(
            "[{}] [WEB-DASHBOARD] {}",
            record.level(),
            record.args()
        );
    }

    fn flush(&self) {}
}

static LOGGER: StderrLogger = StderrLogger;

extern "C" fn signal_handler(_: libc::c_int) {
    RUNNING.store(false, Ordering::SeqCst);
}

#[tokio::main]
async fn main() {
    let _ = log::set_logger(&LOGGER);
    log::set_max_level(log::LevelFilter::Info);

    unsafe {
        libc::signal(
            libc::SIGINT,
            signal_handler as *const () as libc::sighandler_t,
        );
        libc::signal(
            libc::SIGTERM,
            signal_handler as *const () as libc::sighandler_t,
        );
        libc::signal(libc::SIGPIPE, libc::SIG_IGN);
    }

    let options = parse_args();
    let web_root = resolve_web_root(options.web_root);
    let data_dir = default_data_dir();
    let socket_path = options
        .socket_path
        .or_else(|| std::env::var("TIZENCLAW_SOCKET_PATH").ok())
        .unwrap_or_else(|| DEFAULT_SOCKET_NAME.to_string());

    let state = AppState {
        web_root,
        socket_path,
        data_dir,
    };

    let app = Router::new()
        .route("/", get(index_handler))
        .route("/static/*path", get(static_handler))
        .route("/api/status", get(api_status))
        .route("/api/tools", get(api_tools))
        .route("/api/backends", get(api_backends))
        .route("/api/sessions", get(api_sessions))
        .route("/api/prompt", post(api_prompt))
        .route("/api/audit", get(api_audit))
        .route("/api/backend/config", post(api_backend_config))
        .route("/api/reload/tools", post(api_reload_tools))
        .route("/api/reload/backends", post(api_reload_backends))
        .route("/ws/stream", get(ws_stream))
        .fallback(get(not_found_handler))
        .with_state(state);

    let bind_addr = format!("127.0.0.1:{}", options.port);
    let listener = match tokio::net::TcpListener::bind(&bind_addr).await {
        Ok(listener) => listener,
        Err(err) => {
            log::error!("failed to bind {}: {}", bind_addr, err);
            std::process::exit(1);
        }
    };

    log::info!("listening on {}", bind_addr);

    let server = axum::serve(listener, app).with_graceful_shutdown(async {
        let mut interval = tokio::time::interval(Duration::from_millis(250));
        while RUNNING.load(Ordering::SeqCst) {
            interval.tick().await;
        }
    });

    if let Err(err) = server.await {
        log::error!("server error: {}", err);
    }
}

struct CliOptions {
    port: u16,
    web_root: Option<PathBuf>,
    socket_path: Option<String>,
}

fn parse_args() -> CliOptions {
    let mut port = DEFAULT_DASHBOARD_PORT;
    let mut web_root = None;
    let mut socket_path = None;
    let args = std::env::args().skip(1).collect::<Vec<_>>();
    let mut index = 0usize;

    while index < args.len() {
        match args[index].as_str() {
            "--port" if index + 1 < args.len() => {
                port = args[index + 1].parse().unwrap_or(DEFAULT_DASHBOARD_PORT);
                index += 2;
            }
            "--web-root" if index + 1 < args.len() => {
                web_root = Some(expand_home(PathBuf::from(&args[index + 1])));
                index += 2;
            }
            "--socket-path" if index + 1 < args.len() => {
                socket_path = Some(args[index + 1].clone());
                index += 2;
            }
            _ => {
                index += 1;
            }
        }
    }

    CliOptions {
        port,
        web_root,
        socket_path,
    }
}

fn default_data_dir() -> PathBuf {
    if let Ok(path) = std::env::var("TIZENCLAW_DATA_DIR") {
        return expand_home(PathBuf::from(path));
    }
    if is_tizen_runtime() {
        PathBuf::from("/opt/usr/share/tizenclaw")
    } else {
        expand_home(PathBuf::from("~/.tizenclaw"))
    }
}

fn resolve_web_root(cli_web_root: Option<PathBuf>) -> PathBuf {
    if let Some(path) = cli_web_root {
        return path;
    }
    if let Ok(path) = std::env::var("TIZENCLAW_WEB_ROOT") {
        return expand_home(PathBuf::from(path));
    }
    if is_tizen_runtime() {
        return PathBuf::from("/opt/usr/share/tizenclaw/web/");
    }
    expand_home(PathBuf::from("~/.tizenclaw/web/"))
}

fn expand_home(path: PathBuf) -> PathBuf {
    let raw = path.to_string_lossy();
    if raw == "~" {
        return std::env::var("HOME")
            .map(PathBuf::from)
            .unwrap_or_else(|_| PathBuf::from("/tmp"));
    }
    if let Some(suffix) = raw.strip_prefix("~/") {
        let home = std::env::var("HOME").unwrap_or_else(|_| "/tmp".to_string());
        return PathBuf::from(home).join(suffix);
    }
    path
}

fn is_tizen_runtime() -> bool {
    StdPath::new("/etc/tizen-release").exists()
}

async fn index_handler(State(state): State<AppState>) -> impl IntoResponse {
    serve_file(&state.web_root.join("index.html"))
}

async fn static_handler(
    State(state): State<AppState>,
    Path(path): Path<String>,
) -> impl IntoResponse {
    let Some(safe_path) = sanitize_relative_path(&path) else {
        return error_response(StatusCode::BAD_REQUEST, "Invalid static path");
    };
    serve_file(&state.web_root.join("static").join(safe_path))
}

async fn api_status(State(state): State<AppState>) -> impl IntoResponse {
    proxy_jsonrpc(&state.socket_path, "runtime_status", json!({})).await
}

async fn api_tools(State(state): State<AppState>) -> impl IntoResponse {
    proxy_jsonrpc(&state.socket_path, "tool.list", json!({})).await
}

async fn api_backends(State(state): State<AppState>) -> impl IntoResponse {
    proxy_jsonrpc(&state.socket_path, "backend.list", json!({})).await
}

async fn api_sessions(State(state): State<AppState>) -> impl IntoResponse {
    proxy_jsonrpc(&state.socket_path, "session.list", json!({})).await
}

async fn api_prompt(
    State(state): State<AppState>,
    Json(request): Json<PromptRequest>,
) -> impl IntoResponse {
    proxy_jsonrpc(
        &state.socket_path,
        "process_prompt",
        json!({
            "prompt": request.prompt,
            "session_id": request.session_id.unwrap_or_default(),
        }),
    )
    .await
}

async fn api_backend_config(
    State(state): State<AppState>,
    Json(request): Json<BackendConfigRequest>,
) -> impl IntoResponse {
    proxy_jsonrpc(
        &state.socket_path,
        "backend.config.set",
        json!({
            "path": request.path,
            "value": request.value,
        }),
    )
    .await
}

async fn api_reload_tools(State(state): State<AppState>) -> impl IntoResponse {
    proxy_jsonrpc(&state.socket_path, "tool.reload", json!({})).await
}

async fn api_reload_backends(State(state): State<AppState>) -> impl IntoResponse {
    proxy_jsonrpc(&state.socket_path, "backend.reload", json!({})).await
}

async fn api_audit(
    State(state): State<AppState>,
    Query(query): Query<AuditQuery>,
) -> impl IntoResponse {
    let limit = query.limit.unwrap_or(50).clamp(1, 500);
    match load_audit_events(state.data_dir.clone(), limit).await {
        Ok(events) => (StatusCode::OK, Json(json!({ "events": events }))).into_response(),
        Err(err) => error_response(StatusCode::SERVICE_UNAVAILABLE, &err),
    }
}

async fn ws_stream(State(state): State<AppState>, mut request: Request) -> impl IntoResponse {
    if !is_websocket_upgrade(&request) {
        return error_response(StatusCode::BAD_REQUEST, "Expected WebSocket upgrade request");
    }

    let Some(key) = request.headers().get(header::SEC_WEBSOCKET_KEY).cloned() else {
        return error_response(StatusCode::BAD_REQUEST, "Missing Sec-WebSocket-Key header");
    };

    let accept = websocket_accept(&key);
    let on_upgrade = upgrade::on(&mut request);
    let socket_path = state.socket_path.clone();
    tokio::spawn(async move {
        match on_upgrade.await {
            Ok(upgraded) => {
                if let Err(err) = handle_ws_stream(TokioIo::new(upgraded), socket_path).await {
                    log::warn!("websocket stream failed: {}", err);
                }
            }
            Err(err) => {
                log::warn!("websocket upgrade failed: {}", err);
            }
        }
    });

    let mut response = Response::new(Body::empty());
    *response.status_mut() = StatusCode::SWITCHING_PROTOCOLS;
    response
        .headers_mut()
        .insert(header::CONNECTION, HeaderValue::from_static("upgrade"));
    response
        .headers_mut()
        .insert(header::UPGRADE, HeaderValue::from_static("websocket"));
    response.headers_mut().insert(
        header::SEC_WEBSOCKET_ACCEPT,
        HeaderValue::from_str(&accept).unwrap_or_else(|_| HeaderValue::from_static("")),
    );
    response
}

async fn handle_ws_stream(
    mut socket: TokioIo<upgrade::Upgraded>,
    socket_path: String,
) -> Result<(), String> {
    let Some(payload) = read_websocket_text_frame(&mut socket).await? else {
        return Ok(());
    };
    let Ok(request) = serde_json::from_str::<StreamRequest>(&payload) else {
        write_websocket_text_frame(&mut socket, &json!({"error": "Invalid stream request"}).to_string())
            .await?;
        write_websocket_close_frame(&mut socket).await?;
        return Ok(());
    };

    let (tx, mut rx) = tokio::sync::mpsc::channel::<Result<Option<String>, String>>(32);
    let prompt = request.prompt;
    let session_id = request.session_id.unwrap_or_default();

    tokio::task::spawn_blocking(move || {
        let result = stream_prompt_chunks(&socket_path, &prompt, &session_id, |chunk| {
            tx.blocking_send(Ok(Some(chunk.to_string())))
                .map_err(|_| "WebSocket receiver dropped".to_string())
        });

        match result {
            Ok(Some(final_text)) => {
                let _ = tx.blocking_send(Ok(Some(final_text)));
            }
            Ok(None) => {
                let _ = tx.blocking_send(Ok(None));
            }
            Err(err) => {
                let _ = tx.blocking_send(Err(err));
            }
        }
    });

    while let Some(message) = rx.recv().await {
        match message {
            Ok(Some(chunk)) => {
                write_websocket_text_frame(&mut socket, &chunk).await?;
            }
            Ok(None) => {
                write_websocket_close_frame(&mut socket).await?;
                return Ok(());
            }
            Err(err) => {
                write_websocket_text_frame(&mut socket, &json!({ "error": err }).to_string())
                    .await?;
                write_websocket_close_frame(&mut socket).await?;
                return Ok(());
            }
        }
    }

    Ok(())
}

async fn proxy_jsonrpc(socket_path: &str, method: &str, params: Value) -> Response<Body> {
    let socket_path = socket_path.to_string();
    let method = method.to_string();
    match tokio::task::spawn_blocking(move || call_ipc(&socket_path, &method, params)).await {
        Ok(Ok(result)) => (StatusCode::OK, Json(result)).into_response(),
        Ok(Err(err)) => error_response(StatusCode::SERVICE_UNAVAILABLE, &err),
        Err(err) => error_response(
            StatusCode::INTERNAL_SERVER_ERROR,
            &format!("Failed to join IPC task: {}", err),
        ),
    }
}

async fn load_audit_events(data_dir: PathBuf, limit: usize) -> Result<Vec<Value>, String> {
    tokio::task::spawn_blocking(move || query_audit_events(&data_dir, limit))
        .await
        .map_err(|err| format!("Failed to join audit task: {}", err))?
}

fn serve_file(path: &StdPath) -> Response<Body> {
    match std::fs::read(path) {
        Ok(bytes) => {
            let mime = content_type_for_path(path);
            let mut response = Response::new(Body::from(bytes));
            *response.status_mut() = StatusCode::OK;
            response.headers_mut().insert(
                header::CONTENT_TYPE,
                HeaderValue::from_static(mime),
            );
            response
        }
        Err(err) if err.kind() == ErrorKind::NotFound => not_found(),
        Err(err) => error_response(
            StatusCode::INTERNAL_SERVER_ERROR,
            &format!("Failed to read '{}': {}", path.display(), err),
        ),
    }
}

fn sanitize_relative_path(path: &str) -> Option<PathBuf> {
    let candidate = PathBuf::from(path);
    if candidate.is_absolute() {
        return None;
    }
    if candidate
        .components()
        .any(|component| matches!(component, std::path::Component::ParentDir))
    {
        return None;
    }
    Some(candidate)
}

fn content_type_for_path(path: &StdPath) -> &'static str {
    match path.extension().and_then(|ext| ext.to_str()).unwrap_or("") {
        "html" => "text/html; charset=utf-8",
        "js" => "application/javascript; charset=utf-8",
        "css" => "text/css; charset=utf-8",
        "json" => "application/json; charset=utf-8",
        "svg" => "image/svg+xml",
        "png" => "image/png",
        "jpg" | "jpeg" => "image/jpeg",
        "ico" => "image/x-icon",
        _ => "application/octet-stream",
    }
}

fn call_ipc(socket_path: &str, method: &str, params: Value) -> Result<Value, String> {
    let mut stream = connect_socket(socket_path)?;
    let request = json!({
        "jsonrpc": "2.0",
        "id": next_request_id(),
        "method": method,
        "params": params,
    });

    write_frame(&mut stream, &request.to_string())?;

    loop {
        let frame = read_frame(&mut stream)?;
        let payload: Value =
            serde_json::from_str(&frame).map_err(|err| format!("Invalid JSON-RPC frame: {}", err))?;

        if payload.get("method").and_then(Value::as_str) == Some("stream_chunk") {
            continue;
        }

        if let Some(error) = payload.get("error") {
            return Err(error.to_string());
        }

        return Ok(payload.get("result").cloned().unwrap_or(Value::Null));
    }
}

fn stream_prompt_chunks<F>(
    socket_path: &str,
    prompt: &str,
    session_id: &str,
    mut on_chunk: F,
) -> Result<Option<String>, String>
where
    F: FnMut(&str) -> Result<(), String>,
{
    let mut stream = connect_socket(socket_path)?;
    let request = json!({
        "jsonrpc": "2.0",
        "id": next_request_id(),
        "method": "process_prompt_stream",
        "params": {
            "prompt": prompt,
            "session_id": session_id,
        },
    });
    write_frame(&mut stream, &request.to_string())?;

    let mut saw_chunk = false;
    loop {
        let frame = read_frame(&mut stream)?;
        let payload: Value =
            serde_json::from_str(&frame).map_err(|err| format!("Invalid JSON-RPC frame: {}", err))?;

        if payload.get("method").and_then(Value::as_str) == Some("stream_chunk") {
            if let Some(chunk) = payload
                .get("params")
                .and_then(|value| value.get("chunk"))
                .and_then(Value::as_str)
            {
                saw_chunk = true;
                on_chunk(chunk)?;
            }
            continue;
        }

        if let Some(error) = payload.get("error") {
            return Err(error.to_string());
        }

        if saw_chunk {
            return Ok(None);
        }

        let final_text = payload
            .get("result")
            .and_then(|value| value.get("text"))
            .and_then(Value::as_str)
            .map(|value| value.to_string());
        return Ok(final_text);
    }
}

fn query_audit_events(data_dir: &StdPath, limit: usize) -> Result<Vec<Value>, String> {
    let candidate_paths = [
        data_dir.join("audit.db"),
        data_dir.join("audit").join("audit.db"),
        data_dir.join("sessions").join("sessions.db"),
    ];

    for path in candidate_paths {
        if !path.exists() {
            continue;
        }

        let connection =
            rusqlite::Connection::open(&path).map_err(|err| format!("DB open {}: {}", path.display(), err))?;
        let mut statement = match connection.prepare(
            "SELECT id, event_type, session_id, details, timestamp
             FROM audit_events
             ORDER BY timestamp DESC, id DESC
             LIMIT ?1",
        ) {
            Ok(statement) => statement,
            Err(_) => continue,
        };

        let rows = statement
            .query_map([limit as i64], |row| {
                let raw_details: String = row.get(3)?;
                let details =
                    serde_json::from_str(&raw_details).unwrap_or_else(|_| Value::String(raw_details));
                Ok(json!({
                    "id": row.get::<_, i64>(0)?,
                    "event_type": row.get::<_, String>(1)?,
                    "session_id": row.get::<_, String>(2)?,
                    "details": details,
                    "timestamp": row.get::<_, String>(4)?,
                }))
            })
            .map_err(|err| format!("Audit query {}: {}", path.display(), err))?;

        return Ok(rows.filter_map(Result::ok).collect());
    }

    Ok(Vec::new())
}

fn connect_socket(socket_path: &str) -> Result<UnixStream, String> {
    let socket_path = socket_path.trim();
    if socket_path.is_empty() || (!socket_path.starts_with('/') && !socket_path.starts_with('@')) {
        return connect_abstract_socket(if socket_path.is_empty() {
            DEFAULT_SOCKET_NAME
        } else {
            socket_path
        });
    }

    if let Some(path) = socket_path.strip_prefix('@') {
        return connect_abstract_socket(path);
    }

    let stream = UnixStream::connect(socket_path)
        .map_err(|err| format!("Cannot connect to socket {}: {}", socket_path, err))?;
    configure_stream(&stream)?;
    Ok(stream)
}

fn connect_abstract_socket(name: &str) -> Result<UnixStream, String> {
    let fd = unsafe { libc::socket(libc::AF_UNIX, libc::SOCK_STREAM, 0) };
    if fd < 0 {
        return Err(format!(
            "Cannot create IPC socket: {}",
            std::io::Error::last_os_error()
        ));
    }

    let connect_result = unsafe {
        let mut addr: libc::sockaddr_un = std::mem::zeroed();
        addr.sun_family = libc::AF_UNIX as libc::sa_family_t;
        for (index, byte) in name.as_bytes().iter().enumerate() {
            addr.sun_path[index + 1] = *byte as libc::c_char;
        }
        let addr_len =
            (std::mem::size_of::<libc::sa_family_t>() + 1 + name.len()) as libc::socklen_t;
        libc::connect(fd, &addr as *const _ as *const libc::sockaddr, addr_len)
    };

    if connect_result < 0 {
        let error = std::io::Error::last_os_error();
        unsafe {
            libc::close(fd);
        }
        return Err(format!("Cannot connect to socket @{}: {}", name, error));
    }

    let stream = unsafe { UnixStream::from_raw_fd(fd) };
    configure_stream(&stream)?;
    Ok(stream)
}

fn configure_stream(stream: &UnixStream) -> Result<(), String> {
    stream
        .set_read_timeout(Some(IPC_TIMEOUT))
        .map_err(|err| format!("Failed to set read timeout: {}", err))?;
    stream
        .set_write_timeout(Some(IPC_TIMEOUT))
        .map_err(|err| format!("Failed to set write timeout: {}", err))
}

fn write_frame(stream: &mut UnixStream, payload: &str) -> Result<(), String> {
    let bytes = payload.as_bytes();
    if bytes.len() > MAX_IPC_MESSAGE_SIZE {
        return Err(format!("Payload exceeds maximum IPC size: {}", bytes.len()));
    }

    let len = (bytes.len() as u32).to_be_bytes();
    stream
        .write_all(&len)
        .and_then(|_| stream.write_all(bytes))
        .map_err(|err| format!("Failed to write IPC frame: {}", err))
}

fn read_frame(stream: &mut UnixStream) -> Result<String, String> {
    let deadline = Instant::now() + IPC_TIMEOUT;
    let mut len_buf = [0u8; 4];
    read_exact_with_retry(stream, &mut len_buf, deadline, "read length")?;
    let payload_len = u32::from_be_bytes(len_buf) as usize;
    if payload_len == 0 || payload_len > MAX_IPC_MESSAGE_SIZE {
        return Err(format!("Invalid IPC payload size: {}", payload_len));
    }

    let mut payload = vec![0u8; payload_len];
    read_exact_with_retry(stream, &mut payload, deadline, "read body")?;
    String::from_utf8(payload).map_err(|err| format!("Invalid UTF-8 IPC frame: {}", err))
}

fn read_exact_with_retry<R: Read>(
    reader: &mut R,
    buf: &mut [u8],
    deadline: Instant,
    context: &str,
) -> Result<(), String> {
    let mut offset = 0usize;
    while offset < buf.len() {
        match reader.read(&mut buf[offset..]) {
            Ok(0) => {
                return Err(format!(
                    "IPC {} failed: unexpected EOF after {} of {} bytes",
                    context,
                    offset,
                    buf.len()
                ));
            }
            Ok(read) => offset += read,
            Err(err)
                if matches!(
                    err.kind(),
                    ErrorKind::WouldBlock | ErrorKind::TimedOut | ErrorKind::Interrupted
                ) && Instant::now() < deadline =>
            {
                std::thread::sleep(Duration::from_millis(25));
            }
            Err(err) => return Err(format!("IPC {} failed: {}", context, err)),
        }
    }
    Ok(())
}

fn next_request_id() -> u64 {
    REQUEST_ID.fetch_add(1, Ordering::Relaxed)
}

fn is_websocket_upgrade(request: &Request) -> bool {
    request
        .headers()
        .get(header::UPGRADE)
        .and_then(|value| value.to_str().ok())
        .map(|value| value.eq_ignore_ascii_case("websocket"))
        .unwrap_or(false)
        && request
            .headers()
            .get(header::CONNECTION)
            .and_then(|value| value.to_str().ok())
            .map(|value| value.to_ascii_lowercase().contains("upgrade"))
            .unwrap_or(false)
        && request
            .headers()
            .get(header::SEC_WEBSOCKET_VERSION)
            .and_then(|value| value.to_str().ok())
            == Some("13")
}

fn websocket_accept(key: &HeaderValue) -> String {
    let mut raw = key.as_bytes().to_vec();
    raw.extend_from_slice(b"258EAFA5-E914-47DA-95CA-C5AB0DC85B11");
    BASE64.encode(sha1(&raw))
}

async fn read_websocket_text_frame(
    socket: &mut TokioIo<upgrade::Upgraded>,
) -> Result<Option<String>, String> {
    use tokio::io::AsyncReadExt;

    loop {
        let mut header_buf = [0u8; 2];
        socket
            .read_exact(&mut header_buf)
            .await
            .map_err(|err| format!("WebSocket read header failed: {}", err))?;

        let opcode = header_buf[0] & 0x0f;
        let masked = (header_buf[1] & 0x80) != 0;
        let mut payload_len = (header_buf[1] & 0x7f) as usize;
        if payload_len == 126 {
            let mut extended = [0u8; 2];
            socket
                .read_exact(&mut extended)
                .await
                .map_err(|err| format!("WebSocket read length failed: {}", err))?;
            payload_len = u16::from_be_bytes(extended) as usize;
        } else if payload_len == 127 {
            let mut extended = [0u8; 8];
            socket
                .read_exact(&mut extended)
                .await
                .map_err(|err| format!("WebSocket read length failed: {}", err))?;
            payload_len = u64::from_be_bytes(extended) as usize;
        }

        let mut mask = [0u8; 4];
        if masked {
            socket
                .read_exact(&mut mask)
                .await
                .map_err(|err| format!("WebSocket read mask failed: {}", err))?;
        }

        let mut payload = vec![0u8; payload_len];
        socket
            .read_exact(&mut payload)
            .await
            .map_err(|err| format!("WebSocket read payload failed: {}", err))?;

        if masked {
            for (index, byte) in payload.iter_mut().enumerate() {
                *byte ^= mask[index % 4];
            }
        }

        match opcode {
            0x1 => {
                return String::from_utf8(payload)
                    .map(Some)
                    .map_err(|err| format!("WebSocket text frame was not UTF-8: {}", err));
            }
            0x8 => return Ok(None),
            0x9 => continue,
            _ => continue,
        }
    }
}

async fn write_websocket_text_frame(
    socket: &mut TokioIo<upgrade::Upgraded>,
    text: &str,
) -> Result<(), String> {
    write_websocket_frame(socket, 0x1, text.as_bytes()).await
}

async fn write_websocket_close_frame(
    socket: &mut TokioIo<upgrade::Upgraded>,
) -> Result<(), String> {
    write_websocket_frame(socket, 0x8, &[]).await
}

async fn write_websocket_frame(
    socket: &mut TokioIo<upgrade::Upgraded>,
    opcode: u8,
    payload: &[u8],
) -> Result<(), String> {
    use tokio::io::AsyncWriteExt;

    let mut frame = vec![0x80 | opcode];
    match payload.len() {
        len if len < 126 => frame.push(len as u8),
        len if len <= u16::MAX as usize => {
            frame.push(126);
            frame.extend_from_slice(&(len as u16).to_be_bytes());
        }
        len => {
            frame.push(127);
            frame.extend_from_slice(&(len as u64).to_be_bytes());
        }
    }
    frame.extend_from_slice(payload);
    socket
        .write_all(&frame)
        .await
        .map_err(|err| format!("WebSocket write failed: {}", err))
}

fn error_response(status: StatusCode, message: &str) -> Response<Body> {
    (status, Json(json!({ "error": message }))).into_response()
}

fn not_found() -> Response<Body> {
    (StatusCode::NOT_FOUND, Html("Not Found")).into_response()
}

async fn not_found_handler() -> Response<Body> {
    not_found()
}
