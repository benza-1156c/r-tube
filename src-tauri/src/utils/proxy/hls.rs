//! HLS playlist rewriting, segment streaming, and expiry recovery.

use tokio::io::AsyncWriteExt;
use tokio::net::TcpStream;

use super::connection::write_text;
use super::{
    ACCEPT_LANGUAGE, CHROME_USER_AGENT, MANIFEST_REFRESHER, YT_ORIGIN, YT_REFERER, wrap_hls_url,
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
    if !super::is_allowed_stream_host(&url) {
        return write_text(stream, 403, "host not allowed").await;
    }
    let video_id = params.get("vid").cloned().unwrap_or_default();
    let is_master = url.contains("hls_variant");
    let is_playlist = is_master || url.contains("hls_playlist");

    match fetch(client, &url).await {
        Ok(response) if response.status().is_success() => {
            serve_response(stream, client, response, &url, &video_id, is_playlist, method).await
        }
        // Only the master playlist can be re-minted as itself. A variant or
        // segment has to keep its own media; answering it with a master
        // playlist makes the player parse the wrong document and reload.
        // Returning 502 lets hls.js retry, and a genuinely expired URL ends
        // in the player's own recovery, which fetches a fresh manifest.
        _ if is_master => serve_refreshed(stream, client, &video_id, method).await,
        _ => write_text(stream, 502, "upstream rejected the request").await,
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
    method: &str,
) -> Result<bool, Box<dyn std::error::Error + Send + Sync>> {
    let Some(refresher) = MANIFEST_REFRESHER.get() else {
        return write_text(stream, 502, "upstream rejected the request").await;
    };

    let Some(raw) = refresher(video_id.to_string()).await else {
        return write_text(stream, 502, "no fresh playlist").await;
    };
    if !super::is_allowed_stream_host(&raw) {
        return write_text(stream, 502, "no fresh playlist").await;
    }

    match fetch(client, &raw).await {
        Ok(response) => serve_response(stream, client, response, &raw, video_id, true, method).await,
        Err(_) => write_text(stream, 502, "upstream rejected the request").await,
    }
}

async fn serve_response(
    stream: &mut TcpStream,
    client: &reqwest::Client,
    response: reqwest::Response,
    url: &str,
    video_id: &str,
    is_playlist: bool,
    method: &str,
) -> Result<bool, Box<dyn std::error::Error + Send + Sync>> {
    let status = response.status();
    if !status.is_success() {
        return write_text(stream, 502, "upstream rejected the request").await;
    }

    let content_type = response
        .headers()
        .get("content-type")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("")
        .to_string();

    if is_playlist || content_type.contains("mpegurl") {
        return serve_playlist(stream, client, response, url, video_id, method).await;
    }
    serve_segment(stream, client, response, url, &content_type, method).await
}

async fn serve_playlist(
    stream: &mut TcpStream,
    client: &reqwest::Client,
    response: reqwest::Response,
    url: &str,
    video_id: &str,
    method: &str,
) -> Result<bool, Box<dyn std::error::Error + Send + Sync>> {
    // The CDN occasionally closes a keep-alive connection mid-body. Reading
    // the whole playlist (with one retry) before answering keeps a truncated
    // response from reaching the player, where it stalls without an error.
    let text = match response.text().await {
        Ok(text) => text,
        Err(_) => {
            let Ok(retried) = fetch(client, url).await else {
                return write_text(stream, 502, "could not read playlist").await;
            };
            if !retried.status().is_success() {
                return write_text(stream, 502, "could not read playlist").await;
            }
            let Ok(text) = retried.text().await else {
                return write_text(stream, 502, "could not read playlist").await;
            };
            text
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
    client: &reqwest::Client,
    response: reqwest::Response,
    url: &str,
    content_type: &str,
    method: &str,
) -> Result<bool, Box<dyn std::error::Error + Send + Sync>> {
    // Buffer the segment before answering. A streamed response with a
    // Content-Length that the CDN truncates looks successful to the player and
    // leaves hls.js retrying a fragment that can never be parsed.
    let body = match response.bytes().await {
        Ok(body) => body,
        Err(_) => {
            let Ok(retried) = fetch(client, url).await else {
                return write_text(stream, 502, "upstream stream failed").await;
            };
            if !retried.status().is_success() {
                return write_text(stream, 502, "upstream stream failed").await;
            }
            let Ok(body) = retried.bytes().await else {
                return write_text(stream, 502, "upstream stream failed").await;
            };
            body
        }
    };

    let head = format!(
        "HTTP/1.1 200 OK\r\nAccess-Control-Allow-Origin: *\r\nContent-Type: {}\r\nContent-Length: {}\r\nConnection: keep-alive\r\n\r\n",
        if content_type.is_empty() {
            "video/mp4"
        } else {
            content_type
        },
        body.len()
    );
    stream.write_all(head.as_bytes()).await?;

    if method != "HEAD" {
        stream.write_all(&body).await?;
    }
    Ok(true)
}

async fn fetch(client: &reqwest::Client, url: &str) -> Result<reqwest::Response, reqwest::Error> {
    let send = || {
        client
            .get(url)
            .header("User-Agent", CHROME_USER_AGENT)
            .header("Referer", YT_REFERER)
            .header("Origin", YT_ORIGIN)
            .header("Accept", "*/*")
            .header("Accept-Language", ACCEPT_LANGUAGE)
            .header("Accept-Encoding", "identity")
            .send()
    };

    match send().await {
        // A pooled connection that the CDN already closed, or one of its
        // transient 5xx responses, is worth one fresh attempt. Anything else
        // is passed through so expiry handling still sees the real status.
        Err(_) => send().await,
        Ok(response) if response.status().is_server_error() => send().await,
        response => response,
    }
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

    // Captions come from the player response. Leaving the manifest's
    // subtitle group in place makes hls.js add its own text tracks for the
    // same languages, and the player renders every cue twice.
    let mut out = String::with_capacity(text.len() * 2);
    for line in text.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        if trimmed.starts_with("#EXT-X-MEDIA") && trimmed.contains("TYPE=SUBTITLES") {
            continue;
        }
        if trimmed.starts_with("#EXT-X-STREAM-INF") {
            out.push_str(&strip_subtitles_attribute(trimmed));
            out.push('\n');
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

/// Remove the `SUBTITLES="..."` attribute from an `EXT-X-STREAM-INF` line.
fn strip_subtitles_attribute(line: &str) -> String {
    const NEEDLE: &str = "SUBTITLES=\"";
    let Some(pos) = line.find(NEEDLE) else {
        return line.to_string();
    };
    let value_start = pos + NEEDLE.len();
    let Some(value_end) = line[value_start..].find('"') else {
        return line.to_string();
    };
    let end = value_start + value_end + 1;
    let start = line[..pos].rfind(',').unwrap_or(pos);

    let mut out = String::with_capacity(line.len());
    out.push_str(&line[..start]);
    if start == pos {
        // The attribute was first, so drop the comma that follows it.
        out.push_str(line[end..].trim_start_matches(','));
    } else {
        out.push_str(&line[end..]);
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
        force_port(1234);
        let master = "#EXTM3U\n#EXT-X-STREAM-INF:BANDWIDTH=1,RESOLUTION=1920x1080\nhttps://manifest.googlevideo.com/api/manifest/hls_playlist/abc/index.m3u8\n";
        let out = rewrite_playlist(master, "https://manifest.googlevideo.com/x", "vid1");
        assert!(out.starts_with("#EXTM3U"));
        assert!(out.contains("RESOLUTION=1920x1080"));
        assert!(out.contains(&prefix()), "got: {out}");
        assert!(!out.contains("\nhttps://manifest.googlevideo.com"));
    }

    #[test]
    fn strips_the_subtitles_attribute_without_leaving_its_value() {
        let line = "#EXT-X-STREAM-INF:BANDWIDTH=1,CODECS=\"avc1,mp4a.40.2\",AUDIO=\"234\",SUBTITLES=\"vtt\",CLOSED-CAPTIONS=NONE";
        let out = strip_subtitles_attribute(line);
        assert_eq!(
            out,
            "#EXT-X-STREAM-INF:BANDWIDTH=1,CODECS=\"avc1,mp4a.40.2\",AUDIO=\"234\",CLOSED-CAPTIONS=NONE"
        );
        assert!(!out.contains("vtt"), "got: {out}");
    }

    #[test]
    fn strips_hls_subtitle_tracks_from_the_master() {
        force_port(1234);
        let master = "#EXTM3U\n#EXT-X-MEDIA:URI=\"sub/en.m3u8\",TYPE=SUBTITLES,GROUP-ID=\"vtt\",NAME=\"English\"\n#EXT-X-STREAM-INF:BANDWIDTH=1,CODECS=\"avc1,mp4a\",AUDIO=\"233\",SUBTITLES=\"vtt\"\nhttps://manifest.googlevideo.com/x.m3u8\n";
        let out = rewrite_playlist(master, "https://manifest.googlevideo.com/master.m3u8", "vid1");

        assert!(!out.contains("TYPE=SUBTITLES"), "got: {out}");
        assert!(!out.contains("SUBTITLES=\""), "got: {out}");
        assert!(out.contains("AUDIO=\"233\""), "got: {out}");
        assert!(out.contains(&prefix()), "got: {out}");
    }

    #[test]
    fn rewrites_uri_attributes_and_relative_paths() {
        force_port(1234);
        let media = "#EXTM3U\n#EXT-X-KEY:METHOD=AES-128,URI=\"key.bin\"\n#EXTINF:2.0,\nseg1.m4s\n";
        let out = rewrite_playlist(
            media,
            "https://manifest.googlevideo.com/a/b/index.m3u8",
            "vid1",
        );
        assert!(out.contains(&format!("URI=\"{}", prefix())), "got: {out}");
        assert!(out.contains("key.bin"));
        assert!(out.contains("seg1.m4s"));
    }
}
