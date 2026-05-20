use std::io::{Read, Write};
use std::net::{IpAddr, Ipv4Addr, SocketAddr, TcpListener, TcpStream};
use std::time::{Duration as StdDuration, Instant};

use serde::Serialize;
use url::Url;

use crate::error::{AppError, ErrorCode};

const DEFAULT_REDIRECT_PORT: u16 = 56121;
const XAI_CALLBACK_ALLOWED_ORIGINS: [&str; 2] = ["https://accounts.x.ai", "https://auth.x.ai"];

#[derive(Debug, Clone, PartialEq, Eq)]
struct HttpRequest {
    method: String,
    target: String,
    origin: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct CallbackResult {
    pub code: Option<String>,
    pub state: Option<String>,
    pub error: Option<String>,
    pub error_description: Option<String>,
}

pub fn wait_for_callback(
    redirect_uri: &str,
    timeout_seconds: Option<u64>,
) -> Result<CallbackResult, AppError> {
    let url = Url::parse(redirect_uri)
        .map_err(|error| AppError::state_file_invalid(format!("invalid redirect_uri: {error}")))?;
    let host = url.host_str().unwrap_or("127.0.0.1");
    let port = url.port().unwrap_or(DEFAULT_REDIRECT_PORT);
    let callback_path = url.path().to_string();

    let listener = TcpListener::bind((host, port)).map_err(|error| {
        AppError::io(format!(
            "failed to bind callback listener on {host}:{port}: {error}"
        ))
    })?;
    listener
        .set_nonblocking(true)
        .map_err(|error| AppError::io(format!("failed to enable nonblocking listener: {error}")))?;

    let timeout = timeout_seconds.unwrap_or(180);
    let start = Instant::now();
    loop {
        match listener.accept() {
            Ok((mut stream, _addr)) => {
                stream
                    .set_read_timeout(Some(StdDuration::from_secs(5)))
                    .map_err(|error| {
                        AppError::io(format!("failed to set callback read timeout: {error}"))
                    })?;

                let request = parse_http_request(&read_http_request(&mut stream)?)?;
                let callback_url = Url::parse(&format!("http://{host}:{port}{}", request.target))
                    .map_err(|error| {
                    AppError::new(
                        ErrorCode::AuthCallbackTimeout,
                        format!("invalid callback URL: {error}"),
                    )
                })?;
                let cors_origin = allowed_callback_origin(request.origin.as_deref());

                if callback_url.path() != callback_path {
                    let _ = write_text_response(
                        &mut stream,
                        404,
                        "Callback path not found",
                        cors_origin,
                    );
                    continue;
                }

                if request.method == "OPTIONS" {
                    let _ = write_empty_response(&mut stream, 204, cors_origin);
                    continue;
                }

                if request.method != "GET" {
                    let _ = write_text_response(
                        &mut stream,
                        405,
                        "Callback method not allowed",
                        cors_origin,
                    );
                    continue;
                }

                let result = callback_result_from_url(&callback_url);
                if result.code.is_none() && result.error.is_none() {
                    let _ = write_html_response(
                        &mut stream,
                        400,
                        cors_origin,
                        "<html><body><h1>Grok authorization not received.</h1><p>No authorization code was present in this callback URL.</p></body></html>",
                    );
                    continue;
                }

                let body = if result.error.is_some() {
                    "<html><body><h1>Grok authorization failed.</h1><p>You can close this tab.</p></body></html>"
                } else {
                    "<html><body><h1>Grok authorization received.</h1><p>You can close this tab.</p></body></html>"
                };
                let _ = write_html_response(&mut stream, 200, cors_origin, body);
                return Ok(result);
            }
            Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => {
                if start.elapsed() >= StdDuration::from_secs(timeout) {
                    return Err(AppError::new(
                        ErrorCode::AuthCallbackTimeout,
                        format!("timed out waiting for OAuth callback after {timeout} seconds"),
                    ));
                }
                std::thread::sleep(StdDuration::from_millis(100));
            }
            Err(error) => {
                return Err(AppError::io(format!("callback listener failed: {error}")));
            }
        }
    }
}

pub fn parse_manual_callback_input(input: &str) -> Result<CallbackResult, AppError> {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return Err(AppError::new(
            ErrorCode::InvalidArgs,
            "--code must not be empty",
        ));
    }

    if let Some(url) = parse_as_callback_url(trimmed)? {
        return Ok(callback_result_from_url(&url));
    }

    Ok(CallbackResult {
        code: Some(trimmed.to_string()),
        state: None,
        error: None,
        error_description: None,
    })
}

pub fn loopback_redirect_uri(port: Option<u16>) -> String {
    let addr = SocketAddr::from((Ipv4Addr::LOCALHOST, port.unwrap_or(DEFAULT_REDIRECT_PORT)));
    format!("http://{}{}", addr, "/callback")
}

pub fn manual_paste_redirect_uri(port: Option<u16>) -> String {
    format!(
        "http://{}:{}{}",
        IpAddr::V4(Ipv4Addr::LOCALHOST),
        port.unwrap_or(DEFAULT_REDIRECT_PORT),
        "/callback"
    )
}

fn parse_as_callback_url(input: &str) -> Result<Option<Url>, AppError> {
    if input.starts_with("http://") || input.starts_with("https://") {
        return Url::parse(input).map(Some).map_err(|error| {
            AppError::new(
                ErrorCode::InvalidArgs,
                format!("failed to parse callback response: {error}"),
            )
        });
    }
    if input.starts_with("/callback?") {
        return Url::parse(&format!("http://127.0.0.1{input}"))
            .map(Some)
            .map_err(|error| {
                AppError::new(
                    ErrorCode::InvalidArgs,
                    format!("failed to parse callback response: {error}"),
                )
            });
    }
    if input.starts_with('?') || input.starts_with("code=") || input.starts_with("error=") {
        return Url::parse(&format!(
            "http://127.0.0.1/callback?{}",
            input.trim_start_matches('?')
        ))
        .map(Some)
        .map_err(|error| {
            AppError::new(
                ErrorCode::InvalidArgs,
                format!("failed to parse callback response: {error}"),
            )
        });
    }
    Ok(None)
}

fn callback_result_from_url(url: &Url) -> CallbackResult {
    let code = url
        .query_pairs()
        .find(|(key, _)| key == "code")
        .map(|(_, value)| value.to_string());
    let state = url
        .query_pairs()
        .find(|(key, _)| key == "state")
        .map(|(_, value)| value.to_string());
    let error = url
        .query_pairs()
        .find(|(key, _)| key == "error")
        .map(|(_, value)| value.to_string());
    let error_description = url
        .query_pairs()
        .find(|(key, _)| key == "error_description")
        .map(|(_, value)| value.to_string());

    CallbackResult {
        code,
        state,
        error,
        error_description,
    }
}

fn read_http_request(stream: &mut TcpStream) -> Result<String, AppError> {
    let mut request = Vec::new();
    let mut buffer = [0_u8; 1024];
    loop {
        let size = stream
            .read(&mut buffer)
            .map_err(|error| AppError::io(format!("failed to read callback request: {error}")))?;
        if size == 0 {
            break;
        }
        request.extend_from_slice(&buffer[..size]);
        if request.windows(4).any(|window| window == b"\r\n\r\n") || request.len() >= 8192 {
            break;
        }
    }

    Ok(String::from_utf8_lossy(&request).to_string())
}

fn parse_http_request(request: &str) -> Result<HttpRequest, AppError> {
    let mut lines = request.lines();
    let first_line = lines.next().unwrap_or_default();
    let mut parts = first_line.split_whitespace();
    let method = parts.next().unwrap_or_default().to_ascii_uppercase();
    let target = parts.next().unwrap_or_default().to_string();

    if method.is_empty() || target.is_empty() {
        return Err(AppError::new(
            ErrorCode::AuthCallbackTimeout,
            "invalid callback request",
        ));
    }

    let origin = lines
        .take_while(|line| !line.trim().is_empty())
        .find_map(|line| {
            let (name, value) = line.split_once(':')?;
            if name.trim().eq_ignore_ascii_case("origin") {
                return Some(value.trim().to_string());
            }
            None
        });

    Ok(HttpRequest {
        method,
        target,
        origin,
    })
}

fn allowed_callback_origin(origin: Option<&str>) -> Option<&str> {
    let origin = origin?;
    XAI_CALLBACK_ALLOWED_ORIGINS
        .iter()
        .copied()
        .find(|allowed| *allowed == origin)
}

fn write_empty_response(
    stream: &mut TcpStream,
    status: u16,
    cors_origin: Option<&str>,
) -> Result<(), AppError> {
    write_http_response(stream, status, "text/plain; charset=utf-8", "", cors_origin)
}

fn write_text_response(
    stream: &mut TcpStream,
    status: u16,
    body: &str,
    cors_origin: Option<&str>,
) -> Result<(), AppError> {
    write_http_response(
        stream,
        status,
        "text/plain; charset=utf-8",
        body,
        cors_origin,
    )
}

fn write_http_response(
    stream: &mut TcpStream,
    status: u16,
    content_type: &str,
    body: &str,
    cors_origin: Option<&str>,
) -> Result<(), AppError> {
    let status_text = match status {
        204 => "No Content",
        200 => "OK",
        400 => "Bad Request",
        404 => "Not Found",
        405 => "Method Not Allowed",
        _ => "Unknown",
    };
    let mut response = format!(
        "HTTP/1.1 {status} {status_text}\r\nContent-Type: {content_type}\r\nContent-Length: {}\r\nConnection: close\r\n",
        body.len(),
    );
    append_cors_headers(&mut response, cors_origin);
    response.push_str("\r\n");
    response.push_str(body);
    stream
        .write_all(response.as_bytes())
        .map_err(|error| AppError::io(format!("failed to write callback response: {error}")))
}

fn write_html_response(
    stream: &mut TcpStream,
    status: u16,
    cors_origin: Option<&str>,
    body: &str,
) -> Result<(), AppError> {
    write_http_response(
        stream,
        status,
        "text/html; charset=utf-8",
        body,
        cors_origin,
    )
}

fn append_cors_headers(response: &mut String, cors_origin: Option<&str>) {
    if let Some(origin) = cors_origin {
        response.push_str(&format!("Access-Control-Allow-Origin: {origin}\r\n"));
        response.push_str("Access-Control-Allow-Methods: GET, OPTIONS\r\n");
        response.push_str("Access-Control-Allow-Headers: Content-Type\r\n");
        response.push_str("Access-Control-Allow-Private-Network: true\r\n");
        response.push_str("Vary: Origin\r\n");
    }
}

#[cfg(test)]
mod tests {
    use std::io::{Read, Write};
    use std::net::{TcpListener, TcpStream};
    use std::thread;
    use std::time::Duration as StdDuration;

    use super::{
        CallbackResult, loopback_redirect_uri, parse_manual_callback_input, wait_for_callback,
    };

    #[test]
    fn parse_manual_callback_accepts_bare_code() {
        let result = parse_manual_callback_input("manual-code").unwrap();
        assert_eq!(
            result,
            CallbackResult {
                code: Some("manual-code".to_string()),
                state: None,
                error: None,
                error_description: None,
            }
        );
    }

    #[test]
    fn parse_manual_callback_accepts_query_string() {
        let result = parse_manual_callback_input("?code=abc&state=xyz").unwrap();
        assert_eq!(
            result,
            CallbackResult {
                code: Some("abc".to_string()),
                state: Some("xyz".to_string()),
                error: None,
                error_description: None,
            }
        );
    }

    #[test]
    fn wait_for_callback_handles_options_preflight_before_get() {
        let port = reserve_free_port();
        let redirect_uri = loopback_redirect_uri(Some(port));
        let handle = thread::spawn(move || wait_for_callback(&redirect_uri, Some(2)).unwrap());

        let preflight = send_request(
            port,
            "OPTIONS /callback HTTP/1.1\r\nHost: 127.0.0.1\r\nOrigin: https://accounts.x.ai\r\nAccess-Control-Request-Method: GET\r\nAccess-Control-Request-Private-Network: true\r\nConnection: close\r\n\r\n",
        );
        assert!(preflight.contains("HTTP/1.1 204 No Content"));
        assert!(preflight.contains("Access-Control-Allow-Origin: https://accounts.x.ai"));
        assert!(preflight.contains("Access-Control-Allow-Methods: GET, OPTIONS"));
        assert!(preflight.contains("Access-Control-Allow-Headers: Content-Type"));
        assert!(preflight.contains("Access-Control-Allow-Private-Network: true"));

        let callback = send_request(
            port,
            "GET /callback?code=abc&state=xyz HTTP/1.1\r\nHost: 127.0.0.1\r\nOrigin: https://accounts.x.ai\r\nConnection: close\r\n\r\n",
        );
        assert!(callback.contains("HTTP/1.1 200 OK"));
        assert!(callback.contains("Grok authorization received."));

        let result = handle.join().unwrap();
        assert_eq!(
            result,
            CallbackResult {
                code: Some("abc".to_string()),
                state: Some("xyz".to_string()),
                error: None,
                error_description: None,
            }
        );
    }

    #[test]
    fn wait_for_callback_keeps_waiting_after_empty_callback_hit() {
        let port = reserve_free_port();
        let redirect_uri = loopback_redirect_uri(Some(port));
        let handle = thread::spawn(move || wait_for_callback(&redirect_uri, Some(2)).unwrap());

        let empty = send_request(
            port,
            "GET /callback HTTP/1.1\r\nHost: 127.0.0.1\r\nConnection: close\r\n\r\n",
        );
        assert!(empty.contains("HTTP/1.1 400 Bad Request"));
        assert!(empty.contains("Grok authorization not received."));

        let callback = send_request(
            port,
            "GET /callback?code=abc&state=xyz HTTP/1.1\r\nHost: 127.0.0.1\r\nConnection: close\r\n\r\n",
        );
        assert!(callback.contains("HTTP/1.1 200 OK"));

        let result = handle.join().unwrap();
        assert_eq!(result.code.as_deref(), Some("abc"));
        assert_eq!(result.state.as_deref(), Some("xyz"));
    }

    #[test]
    fn parse_manual_callback_accepts_error_callback() {
        let result =
            parse_manual_callback_input("error=access_denied&error_description=denied").unwrap();
        assert_eq!(
            result,
            CallbackResult {
                code: None,
                state: None,
                error: Some("access_denied".to_string()),
                error_description: Some("denied".to_string()),
            }
        );
    }

    fn send_request(port: u16, request: &str) -> String {
        for _ in 0..20 {
            if let Ok(mut stream) = TcpStream::connect(("127.0.0.1", port)) {
                stream.write_all(request.as_bytes()).unwrap();
                let mut response = String::new();
                let _ = stream.read_to_string(&mut response);
                return response;
            }
            thread::sleep(StdDuration::from_millis(50));
        }
        panic!("callback listener did not start on port {port}");
    }

    fn reserve_free_port() -> u16 {
        let listener = TcpListener::bind(("127.0.0.1", 0)).unwrap();
        let port = listener.local_addr().unwrap().port();
        drop(listener);
        port
    }
}
