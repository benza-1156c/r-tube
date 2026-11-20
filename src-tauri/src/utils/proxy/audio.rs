//! Audio relay for adaptive (audio-only) YouTube streams.
//!
//! The browser's `<audio>` element drives this endpoint, so unlike the HLS
//! segments it must be a real HTTP resource: it has to answer `Range` requests
//! with `206 Partial Content` and report the total entity length.
//!
//! Two YouTube quirks shape this module:
//!
//! * A CDN URL rejects a request without a bounded range, and rejects any
//!   single range of 1 MiB or more. Chromium opens media with an *open-ended*
//!   `Range: bytes=0-`, which is exactly the forbidden shape.
//! * URLs expire after a few hours.
//!
//! So a browser request is answered by streaming the requested window as a
//! sequence of small bounded ranges, minting a fresh URL if the CDN rejects
//! one. The client sees one continuous response and never notices.

use std::collections::HashMap;

use futures_util::StreamExt;
use tokio::io::AsyncWriteExt;
use tokio::net::TcpStream;

use super::connection::write_text;
use super::{ACCEPT_LANGUAGE, AUDIO_REFRESHER, CHROME_USER_AGENT, YT_ORIGIN, YT_REFERER};

/// Upper bound for one upstream range request. 512 KiB is comfortably below
/// the 1 MiB ceiling YouTube enforces and keeps seeks responsive.
const CHUNK_BYTES: u64 = 512 * 1024;

/// The part of the resource a browser request asks for.
enum Window {
    Full,
    Partial { start: u64, end: u64 },
    Unsatisfiable,
}

/// Serve one audio stream request, honouring the WebView's `Range` header.
pub(super) async fn serve(
    stream: &mut TcpStream,
    client: &reqwest::Client,
    params: &HashMap<String, String>,
    method: &str,
    headers: &HashMap<String, String>,
) -> Result<bool, Box<dyn std::error::Error + Send + Sync>> {
    let Some(url) = params.get("url").filter(|u| !u.is_empty()).cloned() else {
        return write_text(stream, 400, "missing url").await;
    };
    if !super::is_allowed_stream_host(&url) {
        eprintln!("[audio] rejected host for {url}");
        return write_text(stream, 403, "host not allowed").await;
    }

    let video_id = params.get("vid").cloned().unwrap_or_default();
    let itag: u32 = params
        .get("itag")
        .and_then(|value| value.parse().ok())
        .unwrap_or(0);
    let total: u64 = params
        .get("clen")
        .and_then(|value| value.parse().ok())
        .unwrap_or(0);
    let range = headers.get("range").map(String::as_str);

    // Without a known total the window cannot be bounded, so fall back to a
    // plain relay (which YouTube will reject for open-ended ranges).
    if total == 0 {
        return relay_once(stream, client, &url, &video_id, itag, range, method).await;
    }

    match range_window(range, total) {
        Window::Unsatisfiable => {
            let head = format!(
                "HTTP/1.1 416 Range Not Satisfiable\r\n\
Access-Control-Allow-Origin: *\r\n\
Content-Range: bytes */{total}\r\n\
Content-Length: 0\r\n\
Connection: close\r\n\r\n"
            );
            stream.write_all(head.as_bytes()).await?;
            Ok(false)
        }
        Window::Full => {
            stream_window(stream, client, &url, &video_id, itag, 200, 0, total - 1, total, method)
                .await
        }
        Window::Partial { start, end } => {
            stream_window(stream, client, &url, &video_id, itag, 206, start, end, total, method)
                .await
        }
    }
}

/// Stream `[start, end]` of the resource as a sequence of chunked upstream reads.
#[allow(clippy::too_many_arguments)]
async fn stream_window(
    stream: &mut TcpStream,
    client: &reqwest::Client,
    url: &str,
    video_id: &str,
    itag: u32,
    status: u16,
    start: u64,
    end: u64,
    total: u64,
    method: &str,
) -> Result<bool, Box<dyn std::error::Error + Send + Sync>> {
    let length = end - start + 1;

    // Open the first chunk before writing the head: its response carries the
    // real content type (Opus webm or AAC mp4), which the audio element needs.
    let first_end = (start + CHUNK_BYTES - 1).min(end);
    let Some(first) = open_chunk(client, url, video_id, itag, start, first_end).await else {
        eprintln!("[audio] upstream rejected the first chunk");
        return write_text(stream, 502, "upstream rejected the request").await;
    };

    let content_type = first
        .headers()
        .get("content-type")
        .and_then(|value| value.to_str().ok())
        .unwrap_or("audio/webm")
        .to_string();
    let content_range = (status == 206).then(|| format!("bytes {start}-{end}/{total}"));

    let head = response_head(
        status,
        &content_type,
        Some(&length.to_string()),
        content_range.as_deref(),
        "bytes",
        true,
    );
    stream.write_all(head.as_bytes()).await?;
    if method == "HEAD" {
        return Ok(true);
    }

    if !stream_body(stream, first, first_end - start + 1).await? {
        return Ok(false);
    }

    let mut offset = first_end + 1;
    while offset <= end {
        let chunk_end = (offset + CHUNK_BYTES - 1).min(end);
        let Some(response) = open_chunk(client, url, video_id, itag, offset, chunk_end).await else {
            eprintln!("[audio] upstream rejected chunk at {offset}");
            return Ok(false);
        };
        if !stream_body(stream, response, chunk_end - offset + 1).await? {
            return Ok(false);
        }
        offset = chunk_end + 1;
    }

    Ok(true)
}

/// Fetch one bounded chunk, re-minting the URL once if the CDN rejects it.
async fn open_chunk(
    client: &reqwest::Client,
    url: &str,
    video_id: &str,
    itag: u32,
    start: u64,
    end: u64,
) -> Option<reqwest::Response> {
    let requested = format!("bytes={start}-{end}");
    let mut attempt = fetch(client, url, Some(&requested)).await;

    if !matches!(&attempt, Ok(response) if response.status().is_success()) {
        if let Some(refresher) = AUDIO_REFRESHER.get() {
            if let Some(fresh) = refresher(video_id.to_string(), itag).await {
                if super::is_allowed_stream_host(&fresh) {
                    attempt = fetch(client, &fresh, Some(&requested)).await;
                }
            }
        }
    }

    match attempt {
        Ok(response) if response.status().is_success() => Some(response),
        Ok(response) => {
            eprintln!("[audio] upstream {} for chunk {start}", response.status());
            None
        }
        Err(error) => {
            eprintln!("[audio] chunk {start} failed: {error}");
            None
        }
    }
}

/// Copy exactly `expected` bytes to the client; `false` means the peer left.
async fn stream_body(
    stream: &mut TcpStream,
    response: reqwest::Response,
    expected: u64,
) -> Result<bool, Box<dyn std::error::Error + Send + Sync>> {
    let mut written = 0u64;
    let mut body = response.bytes_stream();
    while let Some(chunk) = body.next().await {
        let Ok(chunk) = chunk else { return Ok(false) };
        written += chunk.len() as u64;
        if stream.write_all(&chunk).await.is_err() {
            return Ok(false);
        }
    }
    if written != expected {
        eprintln!("[audio] short chunk: {written} of {expected} bytes");
        return Ok(false);
    }
    Ok(true)
}

/// Plain pass-through for a request whose total length is unknown.
async fn relay_once(
    stream: &mut TcpStream,
    client: &reqwest::Client,
    url: &str,
    video_id: &str,
    itag: u32,
    range: Option<&str>,
    method: &str,
) -> Result<bool, Box<dyn std::error::Error + Send + Sync>> {
    let response = match fetch(client, url, range).await {
        Ok(response) if response.status().is_success() || response.status().as_u16() == 416 => {
            response
        }
        _ => {
            let Some(refresher) = AUDIO_REFRESHER.get() else {
                return write_text(stream, 502, "upstream rejected the request").await;
            };
            let Some(fresh) = refresher(video_id.to_string(), itag).await else {
                return write_text(stream, 502, "no fresh audio stream").await;
            };
            if !super::is_allowed_stream_host(&fresh) {
                return write_text(stream, 502, "no fresh audio stream").await;
            }
            match fetch(client, &fresh, range).await {
                Ok(response) => response,
                Err(_) => return write_text(stream, 502, "upstream rejected the request").await,
            }
        }
    };

    let status = response.status();
    let content_type = response
        .headers()
        .get("content-type")
        .and_then(|value| value.to_str().ok())
        .unwrap_or("audio/webm")
        .to_string();
    let content_length = response
        .headers()
        .get("content-length")
        .and_then(|value| value.to_str().ok())
        .map(str::to_string);
    let content_range = response
        .headers()
        .get("content-range")
        .and_then(|value| value.to_str().ok())
        .map(str::to_string);
    let keep_alive = content_length.is_some();

    let head = response_head(
        status.as_u16(),
        &content_type,
        content_length.as_deref(),
        content_range.as_deref(),
        "bytes",
        keep_alive,
    );
    stream.write_all(head.as_bytes()).await?;
    if method == "HEAD" {
        return Ok(keep_alive);
    }

    let mut body = response.bytes_stream();
    while let Some(chunk) = body.next().await {
        match chunk {
            Ok(chunk) => {
                if stream.write_all(&chunk).await.is_err() {
                    return Ok(false);
                }
            }
            Err(_) => return Ok(false),
        }
    }
    Ok(keep_alive)
}

/// Map a browser `Range` header onto a satisfiable window of the resource.
fn range_window(range: Option<&str>, total: u64) -> Window {
    let Some(range) = range else {
        return Window::Full;
    };
    let Some(spec) = range.trim().strip_prefix("bytes=") else {
        return Window::Full;
    };
    let first = spec.split(',').next().unwrap_or("").trim();
    let Some((start, end)) = first.split_once('-') else {
        return Window::Full;
    };
    let (start, end) = (start.trim(), end.trim());

    // Suffix range: `bytes=-N` means the last N bytes.
    if start.is_empty() {
        let Ok(suffix) = end.parse::<u64>() else {
            return Window::Full;
        };
        if suffix == 0 || total == 0 {
            return Window::Unsatisfiable;
        }
        let start = total.saturating_sub(suffix);
        return Window::Partial {
            start,
            end: total - 1,
        };
    }

    let Ok(start) = start.parse::<u64>() else {
        return Window::Full;
    };
    if start >= total {
        return Window::Unsatisfiable;
    }
    let end = if end.is_empty() {
        total - 1
    } else {
        end.parse::<u64>()
            .map(|value| value.min(total - 1))
            .unwrap_or(total - 1)
    };
    if end < start {
        return Window::Unsatisfiable;
    }

    Window::Partial { start, end }
}

/// Build the response head. Kept separate so the framing rules are testable.
fn response_head(
    status: u16,
    content_type: &str,
    content_length: Option<&str>,
    content_range: Option<&str>,
    accept_ranges: &str,
    keep_alive: bool,
) -> String {
    let reason = match status {
        200 => "OK",
        206 => "Partial Content",
        400 => "Bad Request",
        403 => "Forbidden",
        404 => "Not Found",
        410 => "Gone",
        416 => "Range Not Satisfiable",
        429 => "Too Many Requests",
        500 => "Internal Server Error",
        502 => "Bad Gateway",
        503 => "Service Unavailable",
        _ => "OK",
    };

    let mut head = format!(
        "HTTP/1.1 {status} {reason}\r\n\
Access-Control-Allow-Origin: *\r\n\
Access-Control-Expose-Headers: Content-Range, Content-Length, Accept-Ranges\r\n\
Content-Type: {content_type}\r\n\
Accept-Ranges: {accept_ranges}\r\n\
Cache-Control: no-store\r\n"
    );
    if let Some(len) = content_length {
        head.push_str(&format!("Content-Length: {len}\r\n"));
    }
    if let Some(range) = content_range {
        head.push_str(&format!("Content-Range: {range}\r\n"));
    }
    head.push_str(if keep_alive {
        "Connection: keep-alive\r\n\r\n"
    } else {
        "Connection: close\r\n\r\n"
    });
    head
}

async fn fetch(
    client: &reqwest::Client,
    url: &str,
    range: Option<&str>,
) -> Result<reqwest::Response, reqwest::Error> {
    let mut request = client
        .get(url)
        .header("User-Agent", CHROME_USER_AGENT)
        .header("Referer", YT_REFERER)
        .header("Origin", YT_ORIGIN)
        .header("Accept", "*/*")
        .header("Accept-Language", ACCEPT_LANGUAGE)
        .header("Accept-Encoding", "identity");
    if let Some(range) = range {
        request = request.header("Range", range);
    }
    request.send().await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn relays_partial_content_framing() {
        let head = response_head(
            206,
            "audio/webm",
            Some("1024"),
            Some("bytes 0-1023/4096"),
            "bytes",
            true,
        );
        assert!(head.starts_with("HTTP/1.1 206 Partial Content\r\n"));
        assert!(head.contains("Content-Range: bytes 0-1023/4096\r\n"));
        assert!(head.contains("Content-Length: 1024\r\n"));
        assert!(head.contains("Accept-Ranges: bytes\r\n"));
        assert!(head.ends_with("Connection: keep-alive\r\n\r\n"));
    }

    #[test]
    fn unknown_length_closes_the_connection() {
        let head = response_head(200, "audio/webm", None, None, "bytes", false);
        assert!(head.contains("Connection: close\r\n\r\n"));
        assert!(!head.contains("\r\nContent-Length:"));
    }

    #[test]
    fn browser_open_ended_range_becomes_the_whole_file() {
        // What Chromium sends for a media element.
        match range_window(Some("bytes=0-"), 4096) {
            Window::Partial { start, end } => {
                assert_eq!((start, end), (0, 4095));
            }
            _ => panic!("expected a full window"),
        }
    }

    #[test]
    fn seek_range_is_clamped_to_the_file() {
        match range_window(Some("bytes=1000-99999"), 4096) {
            Window::Partial { start, end } => assert_eq!((start, end), (1000, 4095)),
            _ => panic!("expected a partial window"),
        }
    }

    #[test]
    fn suffix_and_invalid_ranges() {
        match range_window(Some("bytes=-1024"), 4096) {
            Window::Partial { start, end } => assert_eq!((start, end), (3072, 4095)),
            _ => panic!("expected a suffix window"),
        }
        assert!(matches!(
            range_window(Some("bytes=5000-"), 4096),
            Window::Unsatisfiable
        ));
        assert!(matches!(range_window(None, 4096), Window::Full));
    }
}
