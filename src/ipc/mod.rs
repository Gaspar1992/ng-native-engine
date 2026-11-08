use crate::engine::EngineError;
use std::collections::HashMap;
use std::fs;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};

#[cfg(unix)]
use std::os::unix::net::{UnixListener, UnixStream};

/// Strongly-typed Hot Module Replacement (HMR) IPC message.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HmrMessage {
    Reload { script: String },
    Eval { code: String },
    Ping,
}

/// Server response to an HMR IPC request.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HmrResponse {
    Ok { reloaded: bool },
    EvalResult(String),
    Pong,
    Error(String),
}

impl HmrResponse {
    pub fn to_json(&self) -> String {
        match self {
            Self::Ok { reloaded } => format!(r#"{{"status":"ok","reloaded":{reloaded}}}"#),
            Self::EvalResult(res) => {
                let escaped = escape_json(res);
                format!(r#"{{"status":"ok","result":"{escaped}"}}"#)
            }
            Self::Pong => String::from(r#"{"status":"ok","pong":true}"#),
            Self::Error(msg) => {
                let escaped = escape_json(msg);
                format!(r#"{{"status":"error","message":"{escaped}"}}"#)
            }
        }
    }
}

pub type ClientId = usize;

/// Cross-platform HMR IPC Server.
/// Listens for live bundle updates and evaluation requests on a non-blocking local Unix domain socket.
pub struct HmrIpcServer {
    socket_path: PathBuf,
    #[cfg(unix)]
    listener: UnixListener,
    #[cfg(unix)]
    clients: HashMap<ClientId, UnixStream>,
    in_buffers: HashMap<ClientId, String>,
    next_client_id: ClientId,
}

impl HmrIpcServer {
    /// Binds to a local Unix socket path. Removes stale socket files left over from unclean shutdowns.
    pub fn bind(path: impl AsRef<Path>) -> Result<Self, EngineError> {
        let socket_path = path.as_ref().to_path_buf();

        if let Some(parent) = socket_path.parent() {
            let _ = fs::create_dir_all(parent);
        }

        #[cfg(unix)]
        {
            if socket_path.exists() && UnixStream::connect(&socket_path).is_err() {
                let _ = fs::remove_file(&socket_path);
            }

            let listener = UnixListener::bind(&socket_path)
                .map_err(|e| EngineError::ConfigError(format!("Failed to bind HMR socket: {e}")))?;
            listener.set_nonblocking(true).map_err(|e| {
                EngineError::ConfigError(format!("Failed to set HMR listener non-blocking: {e}"))
            })?;

            Ok(Self {
                socket_path,
                listener,
                clients: HashMap::new(),
                in_buffers: HashMap::new(),
                next_client_id: 1,
            })
        }

        #[cfg(not(unix))]
        {
            Err(EngineError::ConfigError(String::from(
                "Unix domain sockets are not supported on this platform",
            )))
        }
    }

    pub fn socket_path(&self) -> &Path {
        &self.socket_path
    }

    /// Polls for incoming client connections and full newline-delimited HMR messages without blocking.
    pub fn poll_messages(&mut self) -> Vec<(ClientId, HmrMessage)> {
        let mut messages = Vec::new();

        #[cfg(unix)]
        {
            while let Ok((stream, _)) = self.listener.accept() {
                if stream.set_nonblocking(true).is_ok() {
                    let cid = self.next_client_id;
                    self.next_client_id += 1;
                    self.clients.insert(cid, stream);
                    self.in_buffers.insert(cid, String::new());
                }
            }

            let mut disconnected = Vec::new();
            let mut read_buf = [0u8; 4096];

            for (&cid, stream) in &mut self.clients {
                loop {
                    match stream.read(&mut read_buf) {
                        Ok(0) => {
                            disconnected.push(cid);
                            break;
                        }
                        Ok(n) => {
                            if let Ok(chunk) = std::str::from_utf8(&read_buf[..n]) {
                                let buf = self.in_buffers.entry(cid).or_default();
                                buf.push_str(chunk);
                            }
                        }
                        Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => break,
                        Err(_) => {
                            disconnected.push(cid);
                            break;
                        }
                    }
                }

                if let Some(buf) = self.in_buffers.get_mut(&cid) {
                    while let Some(pos) = buf.find('\n') {
                        let line = buf[..pos].trim().to_string();
                        buf.drain(..=pos);
                        if !line.is_empty()
                            && let Some(msg) = parse_hmr_json(&line)
                        {
                            messages.push((cid, msg));
                        }
                    }
                }
            }

            for cid in disconnected {
                self.clients.remove(&cid);
                self.in_buffers.remove(&cid);
            }
        }

        messages
    }

    /// Sends a JSON-framed response back to the specified client.
    pub fn send_response(&mut self, client_id: ClientId, response: &HmrResponse) {
        #[cfg(unix)]
        if let Some(stream) = self.clients.get_mut(&client_id) {
            let mut payload = response.to_json();
            payload.push('\n');
            let _ = stream.write_all(payload.as_bytes());
            let _ = stream.flush();
        }
    }
}

impl Drop for HmrIpcServer {
    fn drop(&mut self) {
        #[cfg(unix)]
        let _ = fs::remove_file(&self.socket_path);
    }
}

/// Client for sending HMR updates from Rust tests or tooling.
pub struct HmrIpcClient {
    #[cfg(unix)]
    stream: UnixStream,
}

impl HmrIpcClient {
    pub fn connect(path: impl AsRef<Path>) -> Result<Self, EngineError> {
        #[cfg(unix)]
        {
            let stream = UnixStream::connect(path.as_ref())
                .map_err(|e| EngineError::ConfigError(format!("Failed to connect to HMR: {e}")))?;
            Ok(Self { stream })
        }
        #[cfg(not(unix))]
        {
            Err(EngineError::ConfigError(String::from(
                "Unix domain sockets are not supported on this platform",
            )))
        }
    }

    pub fn send_reload(&mut self, script: &str) -> Result<(), EngineError> {
        self.send_message(&HmrMessage::Reload {
            script: script.to_string(),
        })
    }

    pub fn send_eval(&mut self, code: &str) -> Result<(), EngineError> {
        self.send_message(&HmrMessage::Eval {
            code: code.to_string(),
        })
    }

    pub fn send_ping(&mut self) -> Result<(), EngineError> {
        self.send_message(&HmrMessage::Ping)
    }

    fn send_message(&mut self, msg: &HmrMessage) -> Result<(), EngineError> {
        #[cfg(unix)]
        {
            let mut json = match msg {
                HmrMessage::Reload { script } => {
                    let esc = escape_json(script);
                    format!(r#"{{"type":"reload","script":"{esc}"}}"#)
                }
                HmrMessage::Eval { code } => {
                    let esc = escape_json(code);
                    format!(r#"{{"type":"eval","code":"{esc}"}}"#)
                }
                HmrMessage::Ping => String::from(r#"{"type":"ping"}"#),
            };
            json.push('\n');
            self.stream
                .write_all(json.as_bytes())
                .map_err(|e| EngineError::ConfigError(e.to_string()))?;
            self.stream
                .flush()
                .map_err(|e| EngineError::ConfigError(e.to_string()))?;
            Ok(())
        }
        #[cfg(not(unix))]
        {
            Err(EngineError::ConfigError(String::from(
                "Unsupported platform",
            )))
        }
    }

    pub fn read_response(&mut self) -> Result<String, EngineError> {
        #[cfg(unix)]
        {
            let mut line = String::new();
            let mut byte = [0u8; 1];
            while self.stream.read_exact(&mut byte).is_ok() {
                if byte[0] == b'\n' {
                    break;
                }
                line.push(byte[0] as char);
            }
            Ok(line)
        }
        #[cfg(not(unix))]
        {
            Err(EngineError::ConfigError(String::from(
                "Unsupported platform",
            )))
        }
    }
}

pub fn parse_hmr_json(line: &str) -> Option<HmrMessage> {
    let msg_type = extract_json_value(line, "type")?;
    match msg_type.as_str() {
        "reload" => {
            let script = extract_json_value(line, "script").unwrap_or_default();
            Some(HmrMessage::Reload { script })
        }
        "eval" => {
            let code = extract_json_value(line, "code").unwrap_or_default();
            Some(HmrMessage::Eval { code })
        }
        "ping" => Some(HmrMessage::Ping),
        _ => None,
    }
}

fn extract_json_value(json: &str, key: &str) -> Option<String> {
    let pattern = format!("\"{key}\"");
    let start = json.find(&pattern)? + pattern.len();
    let rest = &json[start..];
    let colon = rest.find(':')? + 1;
    let after_colon = rest[colon..].trim_start();
    if !after_colon.starts_with('"') {
        return None;
    }
    let after_quote = &after_colon[1..];
    let mut end_idx = 0;
    let mut escaped = false;
    for (i, c) in after_quote.char_indices() {
        if escaped {
            escaped = false;
        } else if c == '\\' {
            escaped = true;
        } else if c == '"' {
            end_idx = i;
            break;
        }
    }
    Some(unescape_json(&after_quote[..end_idx]))
}

fn escape_json(raw: &str) -> String {
    let mut out = String::with_capacity(raw.len() + 16);
    for c in raw.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            other => out.push(other),
        }
    }
    out
}

fn unescape_json(raw: &str) -> String {
    let mut out = String::with_capacity(raw.len());
    let mut chars = raw.chars();
    while let Some(c) = chars.next() {
        if c == '\\' {
            match chars.next() {
                Some('n') => out.push('\n'),
                Some('r') => out.push('\r'),
                Some('t') => out.push('\t'),
                Some('"') => out.push('"'),
                Some('\\') | None => out.push('\\'),
                Some(other) => {
                    out.push('\\');
                    out.push(other);
                }
            }
        } else {
            out.push(c);
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_escaped_json_hmr_messages() {
        let raw = r#"{"type":"reload","script":"text win \"Hello\"\nsignal c 1"}"#;
        let parsed = parse_hmr_json(raw).unwrap();
        assert_eq!(
            parsed,
            HmrMessage::Reload {
                script: String::from("text win \"Hello\"\nsignal c 1"),
            }
        );

        let ping = parse_hmr_json(r#"{"type":"ping"}"#).unwrap();
        assert_eq!(ping, HmrMessage::Ping);
    }

    #[cfg(unix)]
    #[test]
    fn server_client_roundtrip_ping_and_reload() {
        let socket_path = PathBuf::from(format!(
            "/tmp/ng-native-test-hmr-{}.sock",
            std::process::id()
        ));
        let mut server = HmrIpcServer::bind(&socket_path).unwrap();
        assert!(socket_path.exists());

        let mut client = HmrIpcClient::connect(&socket_path).unwrap();
        client.send_ping().unwrap();

        let mut messages = Vec::new();
        for _ in 0..10 {
            messages = server.poll_messages();
            if !messages.is_empty() {
                break;
            }
            std::thread::sleep(std::time::Duration::from_millis(5));
        }

        assert_eq!(messages.len(), 1);
        let (cid, msg) = &messages[0];
        assert_eq!(*msg, HmrMessage::Ping);

        server.send_response(*cid, &HmrResponse::Pong);
        let response = client.read_response().unwrap();
        assert_eq!(response, r#"{"status":"ok","pong":true}"#);

        drop(server);
        assert!(!socket_path.exists());
    }
}
