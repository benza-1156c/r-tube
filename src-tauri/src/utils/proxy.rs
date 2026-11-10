use futures_util::StreamExt;
use std::sync::OnceLock;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;

pub const CHROME_USER_AGENT: &str =
    "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/127.0.0.0 Safari/537.36";
pub const YT_ORIGIN: &str = "https://www.youtube.com";
pub const YT_REFERER: &str = "https://www.youtube.com/";
pub const ACCEPT_LANGUAGE: &str = "en-US,en;q=0.9";

static PROXY_PORT: OnceLock<u16> = OnceLock::new();

pub fn get_proxy_port() -> u16 {
    *PROXY_PORT.get().unwrap_or(&0)
}

pub async fn init_stream_proxy() -> u16 {
    start_proxy_server().await.unwrap_or(0)
}

pub async fn start_proxy_server() -> Result<u16, String> {
    if let Some(&port) = PROXY_PORT.get() {
        return Ok(port);
    }

    let listener = TcpListener::bind("127.0.0.1:0")
        .await
        .map_err(|e| format!("Failed to bind proxy listener: {e}"))?;

    let port = listener
        .local_addr()
        .map_err(|e| format!("Failed to get local port: {e}"))?
        .port();

    let _ = PROXY_PORT.set(port);
    println!("[Stream Proxy] Running on 127.0.0.1:{port}");

    let client = reqwest::Client::builder()
        .pool_max_idle_per_host(20)
        .build()
        .unwrap_or_default();

    tokio::spawn(async move {
        while let Ok((stream, _)) = listener.accept().await {
            let client = client.clone();
            tokio::spawn(async move {
                let _ = stream.set_nodelay(true);
                let _ = handle_connection(stream, client).await;
            });
        }
    });

    Ok(port)
}

pub fn wrap_stream_url(raw_url: &str) -> String {
    if raw_url.starts_with("http://127.0.0.1:") {
        return raw_url.to_string();
    }
    let port = get_proxy_port();
    if port > 0 {
        let encoded: String = url::form_urlencoded::byte_serialize(raw_url.as_bytes()).collect();
        format!("http://127.0.0.1:{port}/stream?url={encoded}")
    } else {
        raw_url.to_string()
    }
}

async fn handle_connection(
    mut stream: tokio::net::TcpStream,
    client: reqwest::Client,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let mut buffer = Vec::with_capacity(1024);
    let mut temp = [0u8; 1024];

    loop {
        let n = stream.read(&mut temp).await?;
        if n == 0 {
            return Ok(());
        }
        buffer.extend_from_slice(&temp[..n]);
        if buffer.windows(4).any(|w| w == b"\r\n\r\n") {
            break;
        }
        if buffer.len() > 32768 {
            break;
        }
    }

    let header_str = String::from_utf8_lossy(&buffer);
    let mut lines = header_str.lines();
    let first_line = match lines.next() {
        Some(l) => l,
        None => return Ok(()),
    };

    let parts: Vec<&str> = first_line.split_whitespace().collect();
    if parts.len() < 2 {
        return Ok(());
    }

    let method = parts[0];
    let path = parts[1];

    if method == "OPTIONS" {
        let cors_resp = "HTTP/1.1 204 No Content\r\n\
Access-Control-Allow-Origin: *\r\n\
Access-Control-Allow-Methods: GET, HEAD, OPTIONS\r\n\
Access-Control-Allow-Headers: *\r\n\
Access-Control-Max-Age: 86400\r\n\
Connection: close\r\n\
Content-Length: 0\r\n\r\n";
        stream.write_all(cors_resp.as_bytes()).await?;
        let _ = stream.shutdown().await;
        return Ok(());
    }

    if method != "GET" && method != "HEAD" {
        let err_resp = "HTTP/1.1 405 Method Not Allowed\r\nConnection: close\r\nContent-Length: 0\r\n\r\n";
        stream.write_all(err_resp.as_bytes()).await?;
        let _ = stream.shutdown().await;
        return Ok(());
    }

    let target_url = if let Some(query_start) = path.find('?') {
        let query = &path[query_start + 1..];
        url::form_urlencoded::parse(query.as_bytes())
            .find(|(k, _)| k == "url")
            .map(|(_, v)| v.into_owned())
    } else {
        None
    };

    let target_url = match target_url {
        Some(u) if !u.is_empty() => u,
        _ => {
            let err_resp = "HTTP/1.1 400 Bad Request\r\nConnection: close\r\nContent-Length: 0\r\n\r\n";
            stream.write_all(err_resp.as_bytes()).await?;
            let _ = stream.shutdown().await;
            return Ok(());
        }
    };

    let mut range_header: Option<String> = None;
    for line in lines {
        if let Some(val) = line.strip_prefix("Range: ") {
            range_header = Some(val.trim().to_string());
            break;
        } else if let Some(val) = line.strip_prefix("range: ") {
            range_header = Some(val.trim().to_string());
            break;
        }
    }

    let build_request = || {
        let mut req = client
            .get(&target_url)
            .header("User-Agent", CHROME_USER_AGENT)
            .header("Referer", YT_REFERER)
            .header("Origin", YT_ORIGIN)
            .header("Accept-Language", ACCEPT_LANGUAGE);
        if let Some(range) = range_header.clone() {
            req = req.header("Range", range);
        }
        req
    };

    let mut resp = match build_request().send().await {
        Ok(r) => r,
        Err(e) => {
            let err_msg = e.to_string();
            let msg = format!(
                "HTTP/1.1 502 Bad Gateway\r\nConnection: close\r\nContent-Length: {}\r\n\r\n{}",
                err_msg.len(),
                err_msg
            );
            let _ = stream.write_all(msg.as_bytes()).await;
            let _ = stream.shutdown().await;
            return Ok(());
        }
    };

    if resp.status() == reqwest::StatusCode::FORBIDDEN {
        tokio::time::sleep(std::time::Duration::from_millis(350)).await;
        if let Ok(retry) = build_request().send().await {
            resp = retry;
        }
    }

    let status = resp.status();
    let mut head = format!(
        "HTTP/1.1 {} {}\r\nConnection: close\r\nAccess-Control-Allow-Origin: *\r\nAccess-Control-Expose-Headers: Content-Length, Content-Range, Content-Type, Accept-Ranges\r\n",
        status.as_u16(),
        status.canonical_reason().unwrap_or("OK")
    );

    if let Some(ct) = resp.headers().get("content-type") {
        if let Ok(v) = ct.to_str() {
            head.push_str(&format!("Content-Type: {}\r\n", v));
        }
    }
    if let Some(cl) = resp.headers().get("content-length") {
        if let Ok(v) = cl.to_str() {
            head.push_str(&format!("Content-Length: {}\r\n", v));
        }
    }
    if let Some(cr) = resp.headers().get("content-range") {
        if let Ok(v) = cr.to_str() {
            head.push_str(&format!("Content-Range: {}\r\n", v));
        }
    }
    if let Some(ar) = resp.headers().get("accept-ranges") {
        if let Ok(v) = ar.to_str() {
            head.push_str(&format!("Accept-Ranges: {}\r\n", v));
        }
    } else {
        head.push_str("Accept-Ranges: bytes\r\n");
    }
    head.push_str("\r\n");

    stream.write_all(head.as_bytes()).await?;

    if method == "HEAD" {
        let _ = stream.shutdown().await;
        return Ok(());
    }

    let mut byte_stream = resp.bytes_stream();
    while let Some(chunk_res) = byte_stream.next().await {
        match chunk_res {
            Ok(chunk) => {
                if stream.write_all(&chunk).await.is_err() {
                    break;
                }
            }
            Err(_) => break,
        }
    }
    let _ = stream.shutdown().await;

    Ok(())
}
