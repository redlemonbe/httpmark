use std::sync::Arc;
use std::time::{Duration, Instant};

use anyhow::Result;
use bytes::Bytes;
use http_body_util::{BodyExt, Full};
use hyper::Request;
use hyper_rustls::HttpsConnectorBuilder;
use hyper_util::client::legacy::Client;
use hyper_util::rt::TokioExecutor;
use tokio::time::sleep;

use crate::cli::Args;
use crate::stats::Stats;

pub async fn run_worker(
    args:     Arc<Args>,
    stats:    Arc<Stats>,
    start:    Instant,
    duration: Duration,
    ramp:     Duration,
) {
    let client = match build_client(&args) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("worker init error: {e}");
            return;
        }
    };

    while !stats.is_stopped() && start.elapsed() < duration {
        // QPS throttle with ramp
        if args.qps > 0 {
            let elapsed = start.elapsed();
            let qps = if ramp.is_zero() || elapsed >= ramp {
                args.qps as f64
            } else {
                args.qps as f64 * (elapsed.as_secs_f64() / ramp.as_secs_f64())
            };

            let delay_us = (1_000_000.0 / qps / args.connections as f64) as u64;
            if delay_us > 0 {
                sleep(Duration::from_micros(delay_us)).await;
            }
        }

        let req_start = Instant::now();
        match send_request(&client, &args).await {
            Ok((status, body_len)) => {
                let latency_us = req_start.elapsed().as_micros() as u64;
                stats.record(latency_us, body_len as u64, status);
            }
            Err(_) => {
                stats.record_error();
            }
        }
    }
}

type BoxClient = Client<hyper_rustls::HttpsConnector<hyper_util::client::legacy::connect::HttpConnector>, Full<Bytes>>;

fn build_client(args: &Args) -> Result<BoxClient> {
    let mut roots = rustls::RootCertStore::empty();

    if args.tls_skip_verify {
        // Custom verifier that accepts all certs
        let config = rustls::ClientConfig::builder()
            .dangerous()
            .with_custom_certificate_verifier(Arc::new(NoVerifier))
            .with_no_client_auth();
        let https = HttpsConnectorBuilder::new()
            .with_tls_config(config)
            .https_or_http()
            .enable_http1();
        let https = if args.http2 {
            https.enable_http2().build()
        } else {
            https.build()
        };
        return Ok(Client::builder(TokioExecutor::new()).build(https));
    }

    // Load native roots
    for cert in rustls_native_certs::load_native_certs().certs {
        let _ = roots.add(cert);
    }
    let mut config = rustls::ClientConfig::builder()
        .with_root_certificates(roots)
        .with_no_client_auth();

    if args.http2 {
        config.alpn_protocols = vec![b"h2".to_vec(), b"http/1.1".to_vec()];
    }

    let https = HttpsConnectorBuilder::new()
        .with_tls_config(config)
        .https_or_http()
        .enable_http1();
    let https = if args.http2 {
        https.enable_http2().build()
    } else {
        https.build()
    };

    Ok(Client::builder(TokioExecutor::new()).build(https))
}

async fn send_request(client: &BoxClient, args: &Args) -> Result<(u16, usize)> {
    let body = match &args.body {
        Some(data) => {
            let bytes = if let Some(path) = data.strip_prefix('@') {
                Bytes::from(std::fs::read(path)?)
            } else {
                Bytes::from(data.clone().into_bytes())
            };
            Full::new(bytes)
        }
        None => Full::new(Bytes::new()),
    };

    let mut builder = Request::builder()
        .method(args.method_http())
        .uri(&args.url);

    for (name, value) in args.parsed_headers() {
        builder = builder.header(&name, &value);
    }

    if args.body.is_some() {
        builder = builder.header("content-type", &args.content_type);
    }

    let req = builder.body(body)?;
    let resp = client.request(req).await?;
    let status = resp.status().as_u16();
    let body = resp.into_body().collect().await?.to_bytes();
    Ok((status, body.len()))
}

// ── TLS no-verify ─────────────────────────────────────────────────────────────

#[derive(Debug)]
struct NoVerifier;

impl rustls::client::danger::ServerCertVerifier for NoVerifier {
    fn verify_server_cert(
        &self, _: &rustls::pki_types::CertificateDer,
        _: &[rustls::pki_types::CertificateDer],
        _: &rustls::pki_types::ServerName, _: &[u8],
        _: rustls::pki_types::UnixTime,
    ) -> Result<rustls::client::danger::ServerCertVerified, rustls::Error> {
        Ok(rustls::client::danger::ServerCertVerified::assertion())
    }

    fn verify_tls12_signature(
        &self, message: &[u8], cert: &rustls::pki_types::CertificateDer,
        dsa: &rustls::DigitallySignedStruct,
    ) -> Result<rustls::client::danger::HandshakeSignatureValid, rustls::Error> {
        rustls::crypto::verify_tls12_signature(message, cert, dsa,
            &rustls::crypto::ring::default_provider().signature_verification_algorithms)
    }

    fn verify_tls13_signature(
        &self, message: &[u8], cert: &rustls::pki_types::CertificateDer,
        dsa: &rustls::DigitallySignedStruct,
    ) -> Result<rustls::client::danger::HandshakeSignatureValid, rustls::Error> {
        rustls::crypto::verify_tls13_signature(message, cert, dsa,
            &rustls::crypto::ring::default_provider().signature_verification_algorithms)
    }

    fn supported_verify_schemes(&self) -> Vec<rustls::SignatureScheme> {
        rustls::crypto::ring::default_provider()
            .signature_verification_algorithms.supported_schemes()
    }
}
