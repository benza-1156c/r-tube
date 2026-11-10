mod connection;
mod hls;

use std::future::Future;
use std::pin::Pin;
use std::sync::{Arc, OnceLock};
use std::time::Duration;
use tokio::net::TcpListener;

pub const CHROME_USER_AGENT: &str = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/127.0.0.0 Safari/537.36";
pub const YT_ORIGIN: &str = "https://www.youtube.com";
pub const YT_REFERER: &str = "https://www.youtube.com/";
pub const ACCEPT_LANGUAGE: &str = "en-US,en;q=0.9";

/// How long a keep-alive connection may sit idle before it is dropped.
pub(super) const IDLE_TIMEOUT: Duration = Duration::from_secs(120);
/// Upper bound for the request head we accept from the WebView.
pub(super) const MAX_HEAD_BYTES: usize = 64 * 1024;

static PROXY_PORT: OnceLock<u16> = OnceLock::new();
static HTTP_CLIENT: OnceLock<reqwest::Client> = OnceLock::new();

pub type RefreshFuture = Pin<Box<dyn Future<Output = Option<String>> + Send>>;
type ManifestRefresher = dyn Fn(String) -> RefreshFuture + Send + Sync + 'static;

static MANIFEST_REFRESHER: OnceLock<Arc<ManifestRefresher>> = OnceLock::new();

/// Register the callback that re-mints an HLS master playlist URL.
///
/// Playlists expire after a few hours, and the proxy needs a fresh one to keep
/// serving a video that is still open.
pub fn set_manifest_refresher<F>(refresher: F)
where
    F: Fn(String) -> RefreshFuture + Send + Sync + 'static,
{
    let _ = MANIFEST_REFRESHER.set(Arc::new(refresher));
}

pub fn get_proxy_port() -> u16 {
    *PROXY_PORT.get().unwrap_or(&0)
}

/// Publish a port for behaviour tests that never start a listener.
#[cfg(test)]
pub(crate) fn force_port(port: u16) -> u16 {
    *PROXY_PORT.get_or_init(|| port)
}

/// Start the proxy if it is not running yet and return its port.
pub async fn ensure_proxy() -> Result<u16, String> {
    if let Some(&port) = PROXY_PORT.get() {
        return Ok(port);
    }
    start_proxy_server().await
}

pub async fn start_proxy_server() -> Result<u16, String> {
    if let Some(&port) = PROXY_PORT.get() {
        return Ok(port);
    }

    let listener = TcpListener::bind("127.0.0.1:0")
        .await
        .map_err(|e| format!("Failed to bind stream proxy listener: {e}"))?;
    let bound_port = listener
        .local_addr()
        .map_err(|e| format!("Failed to get local proxy port: {e}"))?
        .port();

    // Another task may have won the race; keep the published port consistent
    // with the listener that actually answers.
    let port = *PROXY_PORT.get_or_init(|| bound_port);
    if port != bound_port {
        return Ok(port);
    }

    println!("[Stream Proxy] Running on 127.0.0.1:{port}");

    let client = HTTP_CLIENT
        .get_or_init(|| {
            reqwest::Client::builder()
                .pool_max_idle_per_host(32)
                .tcp_keepalive(Some(Duration::from_secs(30)))
                .connect_timeout(Duration::from_secs(10))
                .build()
                .unwrap_or_default()
        })
        .clone();

    tokio::spawn(async move {
        loop {
            match listener.accept().await {
                Ok((stream, _)) => {
                    let client = client.clone();
                    tokio::spawn(async move {
                        let _ = stream.set_nodelay(true);
                        let _ = connection::handle_connection(stream, client).await;
                    });
                }
                Err(e) => {
                    // Never stop accepting: the port is already published.
                    eprintln!("[Stream Proxy] accept failed: {e}");
                    tokio::time::sleep(Duration::from_millis(50)).await;
                }
            }
        }
    });

    Ok(port)
}

/// Wrap a raw HLS URL so the player fetches it through this proxy.
pub fn wrap_hls_url(raw_url: &str, video_id: &str) -> String {
    let port = get_proxy_port();
    if port == 0 || raw_url.starts_with("http://127.0.0.1:") {
        return raw_url.to_string();
    }
    let encoded: String = url::form_urlencoded::byte_serialize(raw_url.as_bytes()).collect();
    let vid: String = url::form_urlencoded::byte_serialize(video_id.as_bytes()).collect();
    format!("http://127.0.0.1:{port}/hls?url={encoded}&vid={vid}")
}

/// Append a diagnostic line to `%TEMP%/rtube-proxy.log` and to stdout.
pub(crate) fn log_line(message: &str) {
    println!("[Stream Proxy] {message}");

    let path = std::env::temp_dir().join("rtube-proxy.log");
    if std::fs::metadata(&path)
        .map(|m| m.len() > 2 * 1024 * 1024)
        .unwrap_or(false)
    {
        let _ = std::fs::remove_file(&path);
    }
    if let Ok(mut file) = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&path)
    {
        use std::io::Write;
        let _ = writeln!(
            file,
            "{} {}",
            chrono::Local::now().format("%H:%M:%S%.3f"),
            message
        );
    }
}
