//! Live playback contract tests.
//!
//! These hit YouTube and the loopback proxy, so they live in their own test
//! binary: the unit tests override the proxy port to check playlist rewriting,
//! and that global override would otherwise poison the port this binary needs.
//!
//! Both contracts share one test because the proxy listener is owned by the
//! tokio runtime that started it: a second `#[tokio::test]` would find the port
//! published but the listener already gone.

use r_tube_lib::services::youtube::YoutubeClient;
use r_tube_lib::utils::proxy;

fn probe_video_id() -> String {
    std::env::var("PROBE_VIDEO_ID").unwrap_or_else(|_| "8XXPxLLgwt8".to_string())
}

/// The audio contract: an audio-only URL that the local proxy serves with
/// `206 Partial Content`, which is what makes `<audio>` seekable.
///
/// The first case mirrors Chromium exactly (`Range: bytes=0-`), because that
/// open-ended shape is what the CDN refuses; the proxy has to stitch bounded
/// chunks behind it.
async fn assert_audio_range(client: &YoutubeClient, video_id: &str) {
    let info = client
        .audio_stream(video_id, Some("th"))
        .await
        .expect("Failed to resolve an audio stream");

    assert!(
        info.mime_type.starts_with("audio/"),
        "expected an audio-only format, got {}",
        info.mime_type
    );
    let total = info
        .content_length
        .filter(|length| *length > 0.0)
        .expect("expected a known content length") as u64;

    let proxied = proxy::wrap_audio_url(&info.url, video_id, info.itag, total);
    let http = reqwest::Client::new();

    // Chromium's opening request for a media element.
    let response = http
        .get(&proxied)
        .header("Range", "bytes=0-")
        .send()
        .await
        .expect("audio proxy request failed");

    assert_eq!(
        response.status().as_u16(),
        206,
        "expected 206 Partial Content for an open-ended range"
    );
    assert_eq!(
        response
            .headers()
            .get("content-range")
            .and_then(|value| value.to_str().ok()),
        Some(format!("bytes 0-{}/{total}", total - 1).as_str()),
        "expected the whole file in the Content-Range"
    );
    assert_eq!(
        response
            .headers()
            .get("content-length")
            .and_then(|value| value.to_str().ok()),
        Some(total.to_string().as_str()),
        "expected the whole file length"
    );

    // Reading it end to end proves the chunk stitching works.
    let body = response.bytes().await.expect("audio body");
    assert_eq!(body.len() as u64, total, "expected the complete audio file");

    // A seek: a small bounded window in the middle.
    let response = http
        .get(&proxied)
        .header("Range", "bytes=1024-2047")
        .send()
        .await
        .expect("seek request failed");
    assert_eq!(response.status().as_u16(), 206);
    let bytes = response.bytes().await.expect("seek body");
    assert_eq!(bytes.len(), 1024, "expected exactly the requested range");

    eprintln!(
        "audio: itag {} served {total} bytes as 206 (open-ended) + a 1 KiB seek, captions={}",
        info.itag,
        info.captions.len()
    );
}

/// The video contract: a proxied HLS master playlist, and its segments
/// reachable through the proxy.
async fn assert_hls_proxy(client: &YoutubeClient, video_id: &str) {
    let info = client
        .get_video_info(video_id)
        .await
        .expect("Failed to get video info");

    assert!(!info.title.is_empty(), "expected a title");
    let hls = info
        .hls_manifest_url
        .as_deref()
        .expect("expected an HLS manifest");
    assert!(
        hls.starts_with("http://127.0.0.1:"),
        "HLS manifest must be proxied, got: {hls}"
    );

    let http = reqwest::Client::new();
    let master = http
        .get(hls)
        .send()
        .await
        .expect("proxy request failed")
        .text()
        .await
        .expect("body");
    assert!(master.starts_with("#EXTM3U"), "expected a playlist");
    assert!(
        master.contains("http://127.0.0.1:"),
        "every playlist URI should be rewritten through the proxy"
    );
    for line in master
        .lines()
        .filter(|line| line.starts_with("#EXT-X-STREAM-INF"))
    {
        assert!(
            !line.contains("SUBTITLES") && !line.contains("vtt"),
            "subtitle attribute left a malformed variant: {line}"
        );
    }

    // Fetch one rewritten variant playlist: its segments must resolve too.
    let variant = master
        .lines()
        .find(|line| line.starts_with("http://127.0.0.1:"))
        .expect("a rewritten variant URI");
    let media = http
        .get(variant)
        .send()
        .await
        .expect("variant request failed")
        .text()
        .await
        .expect("variant body");
    assert!(media.starts_with("#EXTM3U"), "expected a media playlist");

    let segment = media
        .lines()
        .find(|line| line.starts_with("http://127.0.0.1:"))
        .expect("a rewritten segment URI");
    let resp = http.get(segment).send().await.expect("segment request");
    assert!(
        resp.status().is_success(),
        "segment request failed: {}",
        resp.status()
    );
    let bytes = resp.bytes().await.expect("segment bytes");
    assert!(bytes.len() > 1000, "segment too small: {}", bytes.len());
    eprintln!("video: HLS master, variant and segment all proxied");

    // Separate audio groups are common in YouTube HLS. A variant can only
    // start if its audio playlist and segment are reachable too.
    if let Some(audio) = master
        .lines()
        .find(|line| line.starts_with("#EXT-X-MEDIA") && line.contains("TYPE=AUDIO"))
    {
        let uri = audio
            .split("URI=\"")
            .nth(1)
            .and_then(|rest| rest.split('"').next())
            .expect("audio playlist URI");
        let audio_media = http
            .get(uri)
            .send()
            .await
            .expect("audio playlist request")
            .text()
            .await
            .expect("audio playlist body");
        assert!(
            audio_media.starts_with("#EXTM3U"),
            "expected an audio playlist"
        );
        let audio_segment = audio_media
            .lines()
            .find(|line| line.starts_with("http://127.0.0.1:"))
            .expect("a rewritten audio segment");
        let audio_resp = http
            .get(audio_segment)
            .send()
            .await
            .expect("audio segment request");
        assert!(
            audio_resp.status().is_success(),
            "audio segment failed: {}",
            audio_resp.status()
        );
        let audio_bytes = audio_resp.bytes().await.expect("audio segment bytes");
        assert!(
            audio_bytes.len() > 1000,
            "audio segment too small: {}",
            audio_bytes.len()
        );
        eprintln!("video: audio playlist and segment proxied");
    }

    // Captions are fetched through Rust too; an error page or XML here would
    // become an invalid <track> in the player.
    if let Some(caption) = info.captions.first() {
        let caption_http = r_tube_lib::utils::http::client();
        let response = caption_http
            .get(&caption.vtt_url)
            .header("User-Agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36")
            .header("Referer", "https://www.youtube.com/")
            .send()
            .await
            .expect("caption request failed");
        assert!(
            response.status().is_success(),
            "caption request failed: {}",
            response.status()
        );
        let body = response.text().await.expect("caption body");
        assert!(
            body.starts_with("WEBVTT"),
            "caption body is not VTT: {:.200}",
            body
        );
        eprintln!("captions: {} loaded as WebVTT", caption.language_code);
    }
}

#[tokio::test]
async fn playback_contract() {
    let client = YoutubeClient::new().await.expect("Failed to create client");
    let video_id = probe_video_id();

    assert_audio_range(&client, &video_id).await;
    assert_hls_proxy(&client, &video_id).await;
}

/// Comments and the channel page are fetched through the same signed client.
#[tokio::test]
async fn comments_and_channel() {
    let client = YoutubeClient::new().await.expect("Failed to create client");
    let video_id = probe_video_id();

    let info = client
        .get_video_info(&video_id)
        .await
        .expect("Failed to get video info");
    let channel_id = info.channel_id.clone();
    assert!(!channel_id.is_empty(), "expected a channel id");

    let comments = client
        .get_comments(&video_id, None)
        .await
        .expect("Failed to fetch comments");
    eprintln!(
        "comments: total={:?} page={}",
        comments.total,
        comments.comments.len()
    );

    let channel = client
        .get_channel_page(&channel_id)
        .await
        .expect("Failed to fetch channel");
    eprintln!(
        "channel: {:?} videos={}",
        channel.info.name,
        channel.videos.len()
    );
}
