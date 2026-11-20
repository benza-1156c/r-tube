//! Shared HTTP client setup.

use std::sync::OnceLock;

static CLIENT: OnceLock<reqwest::Client> = OnceLock::new();

/// reqwest 0.13 verifies TLS through `rustls-platform-verifier`, which on
/// Android requires JNI initialization that this app does not perform. Without
/// it the first HTTPS request panics with `Expect rustls-platform-verifier to
/// be initialized` and the request never completes (the video proxy just hangs).
///
/// Ship Mozilla's bundled root store on Android instead. Every other platform
/// keeps the operating system's verifier.
#[allow(unused_mut)]
pub fn with_bundled_roots(mut builder: reqwest::ClientBuilder) -> reqwest::ClientBuilder {
    #[cfg(target_os = "android")]
    {
        let roots = webpki_root_certs::TLS_SERVER_ROOT_CERTS
            .iter()
            .filter_map(|der| reqwest::Certificate::from_der(der.as_ref()).ok())
            .collect::<Vec<_>>();

        builder = builder.tls_certs_only(roots);
    }

    builder
}

/// The shared client for one-off requests.
///
/// Captions are fetched one request per language, so building a client per
/// call paid for a TLS handshake every time. Reusing one client keeps the
/// connections alive between those requests.
pub fn client() -> reqwest::Client {
    CLIENT
        .get_or_init(|| {
            with_bundled_roots(
                reqwest::Client::builder()
                    .pool_max_idle_per_host(8)
                    .tcp_keepalive(Some(std::time::Duration::from_secs(30))),
            )
            .build()
            .unwrap_or_default()
        })
        .clone()
}
