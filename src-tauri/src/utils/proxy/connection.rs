//! Minimal HTTP/1.1 server: keep-alive request parsing and dispatch.
//!
//! The WebView opens a few connections and issues many sequential requests on
//! each (one per HLS segment), so the connection survives across requests
//! until the peer closes it or goes idle.

use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;

use super::hls;
use super::{IDLE_TIMEOUT, MAX_HEAD_BYTES};

struct RequestHead {
    method: String,
    target: String,
    close_after: bool,
}

pub(super) async fn handle_connection(
    mut stream: TcpStream,
    client: reqwest::Client,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let mut pending: Vec<u8> = Vec::with_capacity(2048);

    loop {
        let head =
            match tokio::time::timeout(IDLE_TIMEOUT, read_head(&mut stream, &mut pending)).await {
                Ok(Ok(Some(head))) => head,
                _ => return Ok(()),
            };

        let req = match parse_head(&head) {
            Some(req) => req,
            None => return Ok(()),
        };

        let keep_alive = match respond(&mut stream, &client, &req).await {
            Ok(keep_alive) => keep_alive && !req.close_after,
            Err(_) => false,
        };

        if !keep_alive {
            let _ = stream.shutdown().await;
            return Ok(());
        }
    }
}

async fn read_head(
    stream: &mut TcpStream,
    pending: &mut Vec<u8>,
) -> std::io::Result<Option<String>> {
    loop {
        if let Some(pos) = pending.windows(4).position(|w| w == b"\r\n\r\n") {
            let head: Vec<u8> = pending.drain(..pos + 4).take(pos).collect();
            return Ok(Some(String::from_utf8_lossy(&head).into_owned()));
        }
        if pending.len() > MAX_HEAD_BYTES {
            return Ok(None);
        }
        let mut buf = [0u8; 2048];
        let n = stream.read(&mut buf).await?;
        if n == 0 {
            return Ok(None);
        }
        pending.extend_from_slice(&buf[..n]);
    }
}

fn parse_head(head: &str) -> Option<RequestHead> {
    let mut lines = head.split("\r\n").filter(|l| !l.is_empty());
    let first = lines.next()?;
    let mut parts = first.split_whitespace();
    let method = parts.next()?.to_string();
    let target = parts.next()?.to_string();

    // Only `Connection` matters to the framing here.
    let close_after = lines
        .filter_map(|line| line.split_once(':'))
        .find(|(name, _)| name.trim().eq_ignore_ascii_case("connection"))
        .map(|(_, value)| value.trim().eq_ignore_ascii_case("close"))
        .unwrap_or(false);

    Some(RequestHead {
        method,
        target,
        close_after,
    })
}

async fn respond(
    stream: &mut TcpStream,
    client: &reqwest::Client,
    req: &RequestHead,
) -> Result<bool, Box<dyn std::error::Error + Send + Sync>> {
    if req.method == "OPTIONS" {
        let cors = "HTTP/1.1 204 No Content\r\n\
Access-Control-Allow-Origin: *\r\n\
Access-Control-Allow-Methods: GET, HEAD, OPTIONS\r\n\
Access-Control-Allow-Headers: *\r\n\
Access-Control-Max-Age: 86400\r\n\
Content-Length: 0\r\n\r\n";
        stream.write_all(cors.as_bytes()).await?;
        return Ok(true);
    }

    if req.method != "GET" && req.method != "HEAD" {
        write_text(stream, 405, "method not allowed").await?;
        return Ok(true);
    }

    if req.target.starts_with("/hls?") {
        let query = req.target.split_once('?').map(|(_, q)| q).unwrap_or("");
        let params: std::collections::HashMap<String, String> =
            url::form_urlencoded::parse(query.as_bytes())
                .into_owned()
                .collect();
        return hls::serve(stream, client, &params, &req.method).await;
    }

    write_text(stream, 404, "not found").await
}

/// Write a plain-text response and close the connection.
pub(super) async fn write_text(
    stream: &mut TcpStream,
    status: u16,
    message: &str,
) -> Result<bool, Box<dyn std::error::Error + Send + Sync>> {
    let body = message.as_bytes();
    let head = format!(
        "HTTP/1.1 {status} {}\r\nContent-Type: text/plain; charset=utf-8\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
        match status {
            400 => "Bad Request",
            404 => "Not Found",
            405 => "Method Not Allowed",
            502 => "Bad Gateway",
            _ => "Error",
        },
        body.len()
    );
    let _ = stream.write_all(head.as_bytes()).await;
    let _ = stream.write_all(body).await;
    Ok(false)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_request_head_case_insensitively() {
        let head = "GET /hls?url=x HTTP/1.1\r\nHost: 127.0.0.1\r\nconnection: close";
        let req = parse_head(head).expect("head");
        assert_eq!(req.method, "GET");
        assert_eq!(req.target, "/hls?url=x");
        assert!(req.close_after);
    }
}
