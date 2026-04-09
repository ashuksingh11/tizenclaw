use serde_json::{json, Value};
use std::io::{Read, Write};
use std::os::fd::FromRawFd;
use std::os::unix::net::UnixStream;
use std::path::Path;
use std::time::Duration;

const MAX_IPC_MESSAGE_SIZE: usize = 10 * 1024 * 1024;
const DEFAULT_SOCKET_NAME: &str = "tizenclaw.sock";

#[derive(Clone, Debug, Default)]
pub struct ClientOptions {
    pub socket_name: Option<String>,
    pub socket_path: Option<String>,
}

#[derive(Clone, Debug)]
pub struct CallResult {
    pub result: Value,
    pub streamed_chunks: Vec<String>,
}

pub struct IpcClient {
    options: ClientOptions,
}

impl IpcClient {
    pub fn new(options: ClientOptions) -> Self {
        Self { options }
    }

    pub fn call(&self, method: &str, params: Value) -> Result<CallResult, String> {
        let mut stream = self.connect()?;
        let request = json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": method,
            "params": params,
        });

        Self::write_frame(&mut stream, &request.to_string())?;

        let mut streamed_chunks = Vec::new();
        loop {
            let frame = Self::read_frame(&mut stream)?;
            let payload: Value =
                serde_json::from_str(&frame).map_err(|err| format!("Invalid JSON-RPC frame: {}", err))?;

            if payload.get("method").and_then(Value::as_str) == Some("stream_chunk") {
                if let Some(chunk) = payload
                    .get("params")
                    .and_then(|value| value.get("chunk"))
                    .and_then(Value::as_str)
                {
                    streamed_chunks.push(chunk.to_string());
                }
                continue;
            }

            if let Some(error) = payload.get("error") {
                let message = error
                    .get("message")
                    .and_then(Value::as_str)
                    .unwrap_or("Unknown JSON-RPC error");
                return Err(message.to_string());
            }

            let result = payload
                .get("result")
                .cloned()
                .ok_or_else(|| "Missing JSON-RPC result".to_string())?;

            return Ok(CallResult {
                result,
                streamed_chunks,
            });
        }
    }

    fn connect(&self) -> Result<UnixStream, String> {
        if let Some(path) = self
            .options
            .socket_path
            .as_deref()
            .filter(|value| !value.trim().is_empty())
        {
            let stream = UnixStream::connect(Path::new(path))
                .map_err(|err| format!("Failed to connect to socket path '{}': {}", path, err))?;
            stream
                .set_read_timeout(Some(Duration::from_secs(30)))
                .map_err(|err| format!("Failed to set read timeout: {}", err))?;
            return Ok(stream);
        }

        let socket_name = self
            .options
            .socket_name
            .clone()
            .or_else(|| std::env::var("TIZENCLAW_IPC_SOCKET_NAME").ok())
            .filter(|value| !value.trim().is_empty())
            .unwrap_or_else(|| DEFAULT_SOCKET_NAME.to_string());

        let fd = unsafe { libc::socket(libc::AF_UNIX, libc::SOCK_STREAM, 0) };
        if fd < 0 {
            return Err("Failed to create IPC socket".into());
        }

        let connect_result = unsafe {
            let mut addr: libc::sockaddr_un = std::mem::zeroed();
            addr.sun_family = libc::AF_UNIX as libc::sa_family_t;
            for (index, byte) in socket_name.as_bytes().iter().enumerate() {
                addr.sun_path[index + 1] = *byte as libc::c_char;
            }
            let addr_len = (std::mem::size_of::<libc::sa_family_t>() + 1 + socket_name.len())
                as libc::socklen_t;
            libc::connect(fd, &addr as *const _ as *const libc::sockaddr, addr_len)
        };

        if connect_result < 0 {
            let error = std::io::Error::last_os_error();
            unsafe {
                libc::close(fd);
            }
            return Err(format!(
                "Failed to connect to daemon socket '{}': {}",
                socket_name, error
            ));
        }

        let stream = unsafe { UnixStream::from_raw_fd(fd) };
        stream
            .set_read_timeout(Some(Duration::from_secs(30)))
            .map_err(|err| format!("Failed to set read timeout: {}", err))?;
        Ok(stream)
    }

    fn write_frame(stream: &mut UnixStream, payload: &str) -> Result<(), String> {
        let bytes = payload.as_bytes();
        if bytes.len() > MAX_IPC_MESSAGE_SIZE {
            return Err(format!(
                "Payload exceeds maximum IPC size: {} bytes",
                bytes.len()
            ));
        }

        let len = (bytes.len() as u32).to_be_bytes();
        stream
            .write_all(&len)
            .and_then(|_| stream.write_all(bytes))
            .map_err(|err| format!("Failed to write IPC frame: {}", err))
    }

    fn read_frame(stream: &mut UnixStream) -> Result<String, String> {
        let mut len_buf = [0u8; 4];
        stream
            .read_exact(&mut len_buf)
            .map_err(|err| format!("Failed to read IPC frame size: {}", err))?;
        let payload_len = u32::from_be_bytes(len_buf) as usize;
        if payload_len > MAX_IPC_MESSAGE_SIZE {
            return Err(format!("Received oversized IPC frame: {}", payload_len));
        }

        let mut payload = vec![0u8; payload_len];
        stream
            .read_exact(&mut payload)
            .map_err(|err| format!("Failed to read IPC frame body: {}", err))?;
        String::from_utf8(payload).map_err(|err| format!("Invalid UTF-8 IPC frame: {}", err))
    }
}
