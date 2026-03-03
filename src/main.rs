mod extractor;
mod logger;
mod manifest;
mod metrics;
mod utils;

use crate::extractor::LogoPriority;
use crate::metrics::Metrics;
use std::sync::atomic::Ordering;
use std::sync::Arc;
use tokio::io::{self, AsyncBufReadExt, BufReader};
use tokio::sync::{mpsc, Semaphore};

async fn process_domain(
    client: Arc<reqwest::Client>,
    domain: String,
    metrics: Arc<Metrics>,
) -> (String, Option<String>) {
    let url = format!("https://{}", domain);

    metrics.total.fetch_add(1, Ordering::Relaxed);

    match client.get(&url).send().await {
        Ok(response) => {
            if !response.status().is_success() {
                logger::warn!("[{}] HTTP {}", domain, response.status());
                metrics.http_errors.fetch_add(1, Ordering::Relaxed);
                return (domain, None);
            }

            let html = response.text().await.unwrap_or_default();
            let mut data = extractor::extract_site_data(&html, &url);

            if let Some(ref manifest_url) = data.manifest_url {
                let should_upgrade = matches!(
                    data.priority,
                    None | Some(LogoPriority::LargeIcon)
                         | Some(LogoPriority::SquareOgImage)
                         | Some(LogoPriority::SchemaImage)
                );

                if should_upgrade {
                    if let Some(manifest_logo) =
                        manifest::fetch_manifest_icon(&client, manifest_url, &url).await
                    {
                        data.logo_url = Some(manifest_logo);
                        data.priority = Some(LogoPriority::ManifestIcon);
                    }
                }
            }

            if let Some(ref _logo) = data.logo_url {
                metrics.logos_found.fetch_add(1, Ordering::Relaxed);
                if let Some(prio) = data.priority {
                    logger::info!("[{}] {:?}", domain, prio);
                }
            }

            (domain, data.logo_url)
        }
        Err(e) => {
            logger::error!("[{}] {}", domain, e);
            metrics.network_errors.fetch_add(1, Ordering::Relaxed);
            (domain, None)
        }
    }
}

fn parse_args() -> usize {
    let args: Vec<String> = std::env::args().collect();
    let mut concurrency = 50;
    let mut i = 1;

    while i < args.len() {
        match args[i].as_str() {
            "-c" | "--concurrency" => {
                if let Some(val) = args.get(i + 1).and_then(|s| s.parse().ok()) {
                    concurrency = val;
                }
                i += 1;
            }
            "-q" | "--quiet" => logger::mute(),
            _ => {}
        }
        i += 1;
    }

    concurrency
}

#[tokio::main]
async fn main() -> io::Result<()> {
    let concurrency = parse_args();

    let client = Arc::new(
        reqwest::Client::builder()
            .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36")
            .timeout(std::time::Duration::from_secs(10))
            .build()
            .expect("Failed to build client"),
    );

    let semaphore = Arc::new(Semaphore::new(concurrency));
    let metrics = Arc::new(Metrics::default());
    let (tx, mut rx) = mpsc::channel::<(String, Option<String>)>(100);

    let printer = tokio::spawn(async move {
        println!("domain,logo_url");
        while let Some((domain, logo)) = rx.recv().await {
            println!("{},{}", domain, logo.unwrap_or_default());
        }
    });

    let mut sigint = std::pin::pin!(tokio::signal::ctrl_c());
    let reader = BufReader::new(io::stdin());
    let mut lines = reader.lines();

    loop {
        tokio::select! {
            result = lines.next_line() => {
                let line = match result {
                    Ok(Some(l)) => l,
                    Ok(None) => break,
                    Err(e) => {
                        logger::error!("stdin: {}", e);
                        break;
                    }
                };

                let domain = line.trim().to_string();
                if domain.is_empty() {
                    continue;
                }

                // unwrap is ok, it cannot panic since i never close it
                // will automatically backpressure the input if we have too many ongoing tasks
                let permit = Arc::clone(&semaphore).acquire_owned().await.unwrap();
                let client = Arc::clone(&client);
                let metrics = Arc::clone(&metrics);
                let tx = tx.clone();

                tokio::spawn(async move {
                    let _permit = permit;
                    let result = process_domain(client, domain, metrics).await;
                    let _ = tx.send(result).await;
                });
            }

            _ = &mut sigint => {
                eprintln!("\nSIGINT received. Finishing ongoing requests...\n");
                logger::mute();
                break;
            }
        }
    }

    drop(tx);
    let _ = printer.await;

    metrics.log_summary();

    Ok(())
}