//! HLS playlist rewriting, segment streaming, and expiry recovery.

use futures_util::StreamExt;
use tokio::io::AsyncWriteExt;
use tokio::net::TcpStream;

use super::connection::write_text;
use super::{
    ACCEPT_LANGUAGE, CHROME_USER_AGENT, MANIFEST_REFRESHER, YT_ORIGIN, YT_REFERER, log_line,
    wrap_hls_url,
};

/// Serve one HLS playlist (rewritten) or stream a media segment through.
pub(super) async fn serve(
    stream: &mut TcpStream,
    client: &reqwest::Client,
    params: &std::collections::HashMap<String, String>,
    method: &str,
) -> Result<bool, Box<dyn std::error::Error + Send + Sync>> {
    let Some(url) = params.get("url").filter(|u| !u.is_empty()).cloned() else {
        return write_text(stream, 400, "missing url").await;
    };
    let video_id = params.get("vid").cloned().unwrap_or_default();
    let is_playlist = url.contains("hls_variant") || url.contains("hls_playlist");

    match fetch(client, &url).await {
        Ok(response) if response.status().is_success() => {
            serve_response(stream, response, &url, &video_id, is_playlist, method).await
        }
        _ => serve_refreshed(stream, client, &video_id, is_playlist, method).await,
    }
}

/// Re-mint the master playlist once, then serve from the fresh one.
///
/// Playlists expire after a few hours; a fresh one also renews every segment
/// URL inside it, which is what keeps a long-open video playable.
async fn serve_refreshed(
    stream: &mut TcpStream,
    client: &reqwest::Client,
    video_id: &str,
    is_playlist: bool,
    method: &str,
) -> Result<bool, Box<dyn std::error::Error + Send + Sync>> {
    let Some(refresher) = MANIFEST_REFRESHER.get() else {
        return write_text(stream, 502, "upstream rejected the request").await;
    };
    log_line(&format!("HLS refreshing master for {video_id}"));

    let Some(raw) = refresher(video_id.to_string()).await else {
        return write_text(stream, 502, "no fresh playlist").await;
    };

    match fetch(client, &raw).await {
        Ok(response) => serve_response(stream, response, &raw, video_id, is_playlist, method).await,
        Err(e) => {
            log_line(&format!("HLS refresh failed: {e}"));
            write_text(stream, 502, "upstream rejected the request").await
        }
    }
}

async fn serve_response(
    stream: &mut TcpStream,
    response: reqwest::Response,
    url: &str,
    video_id: &str,
    is_playlist: bool,
    method: &str,
) -> Result<bool, Box<dyn std::error::Error + Send + Sync>> {
    let status = response.status();
    if !status.is_success() {
        log_line(&format!("HLS upstream {}", status.as_u16()));
        return write_text(stream, 502, "upstream rejected the request").await;
    }

    let content_type = response
        .headers()
        .get("content-type")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("")
        .to_string();

    if is_playlist || content_type.contains("mpegurl") {
        return serve_playlist(stream, response, url, video_id, method).await;
    }
    serve_segment(stream, response, &content_type, method).await
}

async fn serve_playlist(
    stream: &mut TcpStream,
    response: reqwest::Response,
    url: &str,
    video_id: &str,
    method: &str,
) -> Result<bool, Box<dyn std::error::Error + Send + Sync>> {
    let text = match response.text().await {
        Ok(text) => text,
        Err(e) => {
            log_line(&format!("HLS playlist read failed: {e}"));
            return write_text(stream, 502, "could not read playlist").await;
        }
    };
    if !text.trim_start().starts_with("#EXTM3U") {
        return write_text(stream, 502, "expecting an HLS playlist").await;
    }

    let rewritten = rewrite_playlist(&text, url, video_id);
    let head = format!(
        "HTTP/1.1 200 OK\r\nContent-Type: application/vnd.apple.mpegurl\r\nContent-Length: {}\r\nCache-Control: no-store\r\nAccess-Control-Allow-Origin: *\r\nConnection: keep-alive\r\n\r\n",
        rewritten.len()
    );
    stream.write_all(head.as_bytes()).await?;
    if method != "HEAD" {
        stream.write_all(rewritten.as_bytes()).await?;
    }
    Ok(true)
}

async fn serve_segment(
    stream: &mut TcpStream,
    response: reqwest::Response,
    content_type: &str,
    method: &str,
) -> Result<bool, Box<dyn std::error::Error + Send + Sync>> {
    let content_length = response
        .headers()
        .get("content-length")
        .and_then(|v| v.to_str().ok())
        .map(|v| v.to_string());

    let mut head = format!(
        "HTTP/1.1 200 OK\r\nAccess-Control-Allow-Origin: *\r\nContent-Type: {}\r\nConnection: {}\r\n",
        if content_type.is_empty() {
            "video/mp4".to_string()
        } else {
            content_type.to_string()
        },
        if content_length.is_some() {
            "keep-alive"
        } else {
            "close"
        }
    );
    if let Some(len) = &content_length {
        head.push_str(&format!("Content-Length: {len}\r\n"));
    }
    head.push_str("\r\n");
    stream.write_all(head.as_bytes()).await?;

    if method == "HEAD" {
        return Ok(content_length.is_some());
    }

    let mut byte_stream = response.bytes_stream();
    while let Some(chunk) = byte_stream.next().await {
        match chunk {
            Ok(chunk) => {
                if stream.write_all(&chunk).await.is_err() {
                    return Ok(false);
                }
            }
            Err(e) => {
                log_line(&format!("HLS segment error: {e}"));
                return Ok(false);
            }
        }
    }
    Ok(content_length.is_some())
}

async fn fetch(client: &reqwest::Client, url: &str) -> Result<reqwest::Response, reqwest::Error> {
    client
        .get(url)
        .header("User-Agent", CHROME_USER_AGENT)
        .header("Referer", YT_REFERER)
        .header("Origin", YT_ORIGIN)
        .header("Accept", "*/*")
        .header("Accept-Language", ACCEPT_LANGUAGE)
        .header("Accept-Encoding", "identity")
        .send()
        .await
}

/// Rewrite every URI inside an HLS playlist so it comes back through the proxy.
fn rewrite_playlist(text: &str, base_url: &str, video_id: &str) -> String {
    let base = url::Url::parse(base_url).ok();
    let absolute = |raw: &str| -> String {
        match base.as_ref().and_then(|b| b.join(raw).ok()) {
            Some(url) => url.to_string(),
            None => raw.to_string(),
        }
    };

    let mut out = String::with_capacity(text.len() * 2);
    for line in text.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        if let Some(rest) = trimmed.strip_prefix('#') {
            // Tag lines may carry URI="…" attributes (media, keys, init maps).
            let mut rewritten = String::with_capacity(line.len());
            let mut remaining = rest;
            while let Some(pos) = remaining.find("URI=\"") {
                let (before, after) = remaining.split_at(pos + 5);
                rewritten.push_str(before);
                match after.find('"') {
                    Some(end) => {
                        let (uri, tail) = after.split_at(end);
                        rewritten.push_str(&wrap_hls_url(&absolute(uri), video_id));
                        remaining = tail;
                    }
                    None => {
                        remaining = after;
                        break;
                    }
                }
            }
            rewritten.push_str(remaining);
            out.push('#');
            out.push_str(&rewritten);
            out.push('\n');
        } else {
            out.push_str(&wrap_hls_url(&absolute(trimmed), video_id));
            out.push('\n');
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::utils::proxy::force_port;

    fn prefix() -> String {
        format!("http://127.0.0.1:{}/hls?url=", force_port(1234))
    }

    #[test]
    fn rewrites_variant_and_segment_uris() {
        let master = "#EXTM3U\n#EXT-X-STREAM-INF:BANDWIDTH=1,RESOLUTION=1920x1080\nhttps://manifest.googlevideo.com/api/manifest/hls_playlist/abc/index.m3u8\n";
        let out = rewrite_playlist(master, "https://manifest.googlevideo.com/x", "vid1");
        assert!(out.starts_with("#EXTM3U"));
        assert!(out.contains("RESOLUTION=1920x1080"));
        assert!(out.contains(&prefix()));
        assert!(!out.contains("\nhttps://manifest.googlevideo.com"));
    }

    #[test]
    fn rewrites_uri_attributes_and_relative_paths() {
        let media = "#EXTM3U\n#EXT-X-KEY:METHOD=AES-128,URI=\"key.bin\"\n#EXTINF:2.0,\nseg1.m4s\n";
        let out = rewrite_playlist(
            media,
            "https://manifest.googlevideo.com/a/b/index.m3u8",
            "vid1",
        );
        assert!(out.contains(&format!("URI=\"{}", prefix())));
        assert!(out.contains("key.bin"));
        assert!(out.contains("seg1.m4s"));
    }
}
