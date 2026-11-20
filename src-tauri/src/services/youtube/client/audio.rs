use std::time::Instant;

use innertube_rs::{Innertube, StreamingFormat};

use crate::error::AppError;
use crate::models::youtube::{AudioStreamInfo, PlayerResponse};

use super::{options_for_client, CachedAudio, YoutubeClient, AUDIO_CACHE_TTL};

impl YoutubeClient {
    /// Best audio-only stream for a video, with metadata for the listen UI.
    ///
    /// The returned URL is the raw CDN URL; callers wrap it for the local proxy.
    pub async fn audio_stream(
        &self,
        video_id: &str,
        language: Option<&str>,
    ) -> Result<AudioStreamInfo, AppError> {
        let key = format!("{video_id}:{}", language.unwrap_or_default());

        if let Some(cached) = self.inner.audio_cache.read().await.get(&key) {
            if cached.at.elapsed() < AUDIO_CACHE_TTL {
                return Ok(cached.info.clone());
            }
        }

        let info = self.resolve_audio(video_id, language, None).await?;
        self.inner.audio_cache.write().await.insert(
            key,
            CachedAudio {
                at: Instant::now(),
                info: info.clone(),
            },
        );
        Ok(info)
    }

    /// Re-mint an expired audio URL for the same track, so a paused song can
    /// resume hours later without switching formats.
    ///
    /// The player response is fetched fresh: the cached one still holds the
    /// URL that just expired.
    pub async fn refresh_audio_url(&self, video_id: &str, itag: u32) -> Option<String> {
        self.resolve_audio(video_id, None, Some(itag))
            .await
            .ok()
            .map(|info| info.url)
    }

    /// Resolve the audio-only stream for a video, plus the metadata the listen
    /// UI shows alongside it.
    async fn resolve_audio(
        &self,
        video_id: &str,
        language: Option<&str>,
        wanted_itag: Option<u32>,
    ) -> Result<AudioStreamInfo, AppError> {
        let picked = self.pick_audio(video_id, language, wanted_itag).await?;
        // Reuse the shared player-response mapping instead of copying the
        // metadata fields (and captions) out by hand.
        let meta = PlayerResponse::from(picked.response);
        let format = picked.format;

        Ok(AudioStreamInfo {
            itag: format.itag,
            url: picked.url,
            mime_type: format.mime_type.clone(),
            bitrate: format.bitrate as f64,
            content_length: format
                .content_length
                .as_deref()
                .and_then(|value| value.parse().ok()),
            duration_ms: format
                .approx_duration_ms
                .as_deref()
                .and_then(|value| value.parse().ok()),
            language: format.language(),
            title: meta.title,
            author: meta.channel_name,
            channel_id: meta.channel_id,
            thumbnail_url: meta.thumbnail_url,
            length_seconds: meta.length_seconds,
            description: meta.description,
            view_count: meta.view_count,
            is_live: meta.is_live,
            captions: meta.captions,
        })
    }

    /// Player response to pick an audio stream from, with the chosen format and
    /// the URL the proxy can serve.
    ///
    /// The generic player chain ends on the iOS client, which only exposes AAC
    /// audio, and most other clients return CDN URLs that refuse any range
    /// except one starting at byte 0 — which makes `<audio>` seeking impossible.
    /// The anonymous VISIONOS client returns Opus 160 kbps *and* URLs that
    /// serve arbitrary byte ranges, so it is the one the listen mode uses.
    async fn pick_audio(
        &self,
        video_id: &str,
        language: Option<&str>,
        wanted_itag: Option<u32>,
    ) -> Result<PickedAudio, AppError> {
        let mut fallback: Option<PickedAudio> = None;

        // The playback response already carries the Opus track and the URLs
        // this mode wants. Reuse it instead of asking YouTube again; listen
        // mode opened directly fetches the same response and keeps it for
        // history reporting.
        if wanted_itag.is_none() {
            let info = match self.cached_info(video_id).await {
                Some(info) => Some(info),
                None => self.playback_info(video_id).await.ok(),
            };
            if let Some(info) = info {
                let signed = self.signed().await?;
                match audio_from_response(&signed, info.player_response, language, None) {
                    Ok(picked) if picked.format.itag == 251 => return Ok(picked),
                    Ok(picked) => fallback = Some(picked),
                    Err(e) => eprintln!("[YouTube] playback audio unusable: {e}"),
                }
            }
        }

        let anon = self.anon().await?;

        for client in ["VISIONOS", "ANDROID_VR", "MWEB", "WEB"] {
            let Ok(info) = anon
                .get_basic_info(video_id, Some(&options_for_client(client)))
                .await
                .inspect_err(|e| eprintln!("[YouTube] audio via {client} failed: {e}"))
            else {
                continue;
            };
            if info.player_response.playability_status.status != "OK" {
                continue;
            }

            match audio_from_response(&anon, info.player_response, language, wanted_itag) {
                // Opus (251) is the point of audio-only mode; VISIONOS has it.
                Ok(picked) if picked.format.itag == 251 => return Ok(picked),
                Ok(picked) => {
                    fallback.get_or_insert(picked);
                }
                Err(e) => eprintln!("[YouTube] audio via {client} refused: {e}"),
            }
        }

        if let Some(picked) = fallback {
            return Ok(picked);
        }

        // Restricted videos may only answer a signed-in session. Their URLs may
        // not be seekable, but playing is better than failing outright.
        let signed = self.signed().await?;
        let info = signed
            .get_basic_info(video_id, None)
            .await
            .map_err(|e| AppError::Internal(format!("Failed to get video info: {e}")))?;
        audio_from_response(&signed, info.player_response, language, wanted_itag)
    }
}

fn playable_audio(format: &StreamingFormat) -> bool {
    format.is_audio_only()
        && !format.is_drc()
        && (format.url.is_some() || format.signature_cipher.is_some() || format.cipher.is_some())
}

/// One player response's chosen audio track and the URL that plays it.
struct PickedAudio {
    response: innertube_rs::PlayerResponse,
    format: StreamingFormat,
    url: String,
}

/// Pick the audio track of a player response and mint a URL the proxy can serve.
///
/// The decipherer and PO token must come from the session that fetched the
/// response, so that client is passed in alongside it.
fn audio_from_response(
    yt: &Innertube,
    response: innertube_rs::PlayerResponse,
    language: Option<&str>,
    wanted_itag: Option<u32>,
) -> Result<PickedAudio, AppError> {
    let format = response
        .streaming_data
        .as_ref()
        .and_then(|streaming| pick_audio_format(&streaming.adaptive_formats, language, wanted_itag))
        .ok_or_else(|| AppError::NotFound("This video has no audio-only stream".to_string()))?;

    // Live/DVR audio arrives as an open-ended chunked stream: there is no
    // length to range over, so it cannot be the lightweight `<audio>` path.
    if format.is_type_otf.unwrap_or(false) {
        return Err(AppError::NotFound(
            "Live streams are not available in audio-only mode".to_string(),
        ));
    }

    let url = innertube_rs::endpoints::player::resolve_stream_url_full(
        &format,
        &yt.player.decipherer,
        yt.session.po_token.as_deref(),
        None,
    )
    .map_err(|e| AppError::Internal(format!("Failed to resolve audio stream: {e}")))?;

    Ok(PickedAudio {
        response,
        format,
        url,
    })
}

/// Pick the audio-only format worth playing.
///
/// Priority:
/// 1. The language the user chose in Settings, when the video has that dub.
/// 2. The uploader's original track (never an auto-dub).
/// 3. Opus 160 kbps (`itag 251`), then AAC 128 kbps (`itag 140`), then the
///    highest bitrate left.
///
/// `"original"` (or an empty value) explicitly asks for the original/default
/// track. DRC variants are skipped because they are loudness-normalised copies.
///
/// The choice is read-only: resolving a stream never writes the selected track
/// back to the settings. If the requested language is unavailable the original
/// still plays, and the user's preference is left untouched.
fn pick_audio_format(
    formats: &[StreamingFormat],
    language: Option<&str>,
    wanted_itag: Option<u32>,
) -> Option<StreamingFormat> {
    // A refresh keeps the track the session was already playing.
    if let Some(itag) = wanted_itag {
        if let Some(found) = formats
            .iter()
            .find(|format| format.itag == itag && playable_audio(format))
        {
            return Some(found.clone());
        }
    }

    let pool: Vec<&StreamingFormat> = formats
        .iter()
        .filter(|format| playable_audio(format))
        .collect();
    if pool.is_empty() {
        return None;
    }

    // The user's explicit choice wins over the uploader's default, but only if
    // that language actually exists on this video.
    if let Some(wanted) = language
        .map(str::trim)
        .filter(|lang| !lang.is_empty() && !lang.eq_ignore_ascii_case("original"))
    {
        let wanted = language_base(wanted);
        let matching: Vec<&StreamingFormat> = pool
            .iter()
            .copied()
            .filter(|format| track_speaks(format, &wanted))
            .collect();
        if !matching.is_empty() {
            return best_audio_format(matching);
        }
        // No dub in that language: fall through to the original track.
    }

    best_audio_format(pool)
}

/// `"th-TH"` / `"th_TH"` → `"th"`, lowercased.
fn language_base(tag: &str) -> String {
    tag.split(['-', '_'])
        .next()
        .unwrap_or(tag)
        .trim()
        .to_ascii_lowercase()
}

/// Whether a format carries audio in `wanted` (a base language code).
fn track_speaks(format: &StreamingFormat, wanted: &str) -> bool {
    if format.language().as_deref().map(language_base).as_deref() == Some(wanted) {
        return true;
    }

    // Some multi-audio responses only tag the language on the audio track id.
    format
        .audio_track
        .as_ref()
        .map(|track| language_base(&track.id) == wanted)
        .unwrap_or(false)
}

/// Prefer an original/default track within `pool`, then the best codec.
fn best_audio_format(pool: Vec<&StreamingFormat>) -> Option<StreamingFormat> {
    pool.into_iter()
        .min_by_key(|format| {
            (
                track_rank(format),
                itag_rank(format.itag),
                std::cmp::Reverse(format.bitrate),
            )
        })
        .cloned()
}

/// Which track of a multi-audio video this is: the uploader's original first,
/// then a plain (non-dub) track, then a language dub — and within one kind, the
/// track the uploader marked as default first.
///
/// Multi-audio videos label the untouched track "… original"; that label beats
/// every dub whatever the bitrates say.
fn track_rank(format: &StreamingFormat) -> (u8, bool) {
    let track = format.audio_track.as_ref();
    let labelled_original = track
        .map(|track| track.display_name.to_ascii_lowercase().contains("original"))
        .unwrap_or(false);
    let kind = if labelled_original {
        0
    } else if format.is_original() {
        1
    } else {
        2
    };
    let not_default = !track.map(|track| track.audio_is_default).unwrap_or(false);
    (kind, not_default)
}

fn itag_rank(itag: u32) -> u8 {
    match itag {
        251 => 0, // Opus 160 kbps, 48 kHz - the best audio YouTube serves
        140 => 1, // AAC 128 kbps
        250 => 2, // Opus 70 kbps
        139 => 3, // AAC 48 kbps
        249 => 4, // Opus 50 kbps
        _ => 5,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use innertube_rs::models::format::AudioTrack;

    fn audio(itag: u32, bitrate: u64) -> StreamingFormat {
        StreamingFormat {
            itag,
            bitrate,
            mime_type: "audio/webm; codecs=\"opus\"".to_string(),
            url: Some(format!("https://example.invalid/audio/{itag}")),
            ..Default::default()
        }
    }

    fn with_track(mut format: StreamingFormat, name: &str, is_default: bool) -> StreamingFormat {
        format.audio_track = Some(AudioTrack {
            audio_is_default: is_default,
            display_name: name.to_string(),
            id: name.to_string(),
        });
        format
    }

    fn with_track_id(
        mut format: StreamingFormat,
        name: &str,
        id: &str,
        is_default: bool,
    ) -> StreamingFormat {
        format.audio_track = Some(AudioTrack {
            audio_is_default: is_default,
            display_name: name.to_string(),
            id: id.to_string(),
        });
        format
    }

    #[test]
    fn prefers_opus_160_over_aac() {
        let formats = vec![audio(140, 129_000), audio(251, 160_000)];
        let picked = pick_audio_format(&formats, None, None).expect("an audio format");
        assert_eq!(picked.itag, 251);
    }

    #[test]
    fn ignores_video_only_formats() {
        let mut video = audio(137, 4_000_000);
        video.mime_type = "video/mp4".to_string();
        let formats = vec![video, audio(140, 129_000)];
        let picked = pick_audio_format(&formats, None, None).expect("an audio format");
        assert_eq!(picked.itag, 140);
    }

    #[test]
    fn refresh_keeps_the_requested_itag() {
        let formats = vec![audio(251, 160_000), audio(140, 129_000)];
        let picked = pick_audio_format(&formats, None, Some(140)).expect("an audio format");
        assert_eq!(picked.itag, 140);
    }

    #[test]
    fn labelled_original_beats_a_higher_bitrate_dub() {
        let dub = with_track(audio(251, 160_000), "English (US)", true);
        let original = with_track(audio(140, 129_000), "Thai original", false);
        let formats = vec![dub, original];

        let picked = pick_audio_format(&formats, Some("th"), None).expect("an audio format");
        assert_eq!(picked.itag, 140, "a dub must never win over the original");
    }

    #[test]
    fn prefers_the_default_track_of_equal_originals() {
        let a = with_track(audio(251, 160_000), "Thai", false);
        let b = with_track(audio(251, 160_001), "Thai", true);
        let formats = vec![a, b];

        let picked = pick_audio_format(&formats, None, None).expect("an audio format");
        assert!(picked.audio_track.unwrap().audio_is_default);
    }

    #[test]
    fn user_language_wins_over_the_original_track() {
        let original = with_track(audio(140, 129_000), "English original", true);
        let dub = with_track_id(audio(251, 160_000), "Thai", "th", false);
        let formats = vec![original, dub];

        let picked = pick_audio_format(&formats, Some("th"), None).expect("an audio format");
        assert_eq!(picked.itag, 251, "the selected dub must be used");
    }

    #[test]
    fn language_tags_are_compared_by_base_code() {
        let original = with_track(audio(140, 129_000), "English original", true);
        let dub = with_track_id(audio(251, 160_000), "Thai", "th-TH", false);
        let formats = vec![original, dub];

        let picked = pick_audio_format(&formats, Some("th-TH"), None).expect("an audio format");
        assert_eq!(picked.itag, 251);
    }

    #[test]
    fn explicit_original_ignores_language_dubs() {
        let original = with_track(audio(140, 129_000), "English original", true);
        let dub = with_track_id(audio(251, 160_000), "Thai", "th", false);
        let formats = vec![original, dub];

        let picked = pick_audio_format(&formats, Some("original"), None).expect("an audio format");
        assert_eq!(picked.itag, 140);
    }

    #[test]
    fn unavailable_language_falls_back_to_the_original() {
        let original = with_track(audio(140, 129_000), "English original", true);
        let dub = with_track_id(audio(251, 160_000), "Thai", "th", false);
        let formats = vec![original, dub];

        let picked = pick_audio_format(&formats, Some("fr"), None).expect("an audio format");
        assert_eq!(picked.itag, 140);
    }
}
