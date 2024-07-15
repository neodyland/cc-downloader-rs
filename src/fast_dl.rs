use anyhow::Context;
use futures_util::{stream::iter, Stream, StreamExt};
use reqwest::{
    header::{ACCEPT_RANGES, CONTENT_LENGTH, RANGE},
    Client, StatusCode,
};
use std::{collections::BTreeMap, io, sync::Arc};
use tokio::sync::{mpsc, Semaphore};
use tokio_stream::wrappers::ReceiverStream;
use tokio_util::bytes::Bytes;

static CONCURRENT_REQUEST_LIMIT: usize = 100;

async fn get_content_length(http_client: &reqwest::Client, url: &str) -> anyhow::Result<u64> {
    let head_response = http_client
        .head(url)
        .send()
        .await
        .context("HEAD request failed")?;

    head_response
        .error_for_status_ref()
        .context("HEAD request returned non-success status code")?;

    let Some(accept_ranges) = head_response.headers().get(ACCEPT_RANGES) else {
        anyhow::bail!("Server doesn't support HTTP range requests (missing ACCEPT_RANGES header)");
    };

    let accept_ranges = String::from_utf8_lossy(accept_ranges.as_bytes());
    if accept_ranges != "bytes" {
        anyhow::bail!(
            "Server doesn't support HTTP range requests (Accept-Ranges = {accept_ranges})"
        );
    }
    let Some(content_length) = head_response.headers().get(CONTENT_LENGTH) else {
        anyhow::bail!("HEAD response did not contain a Content-Length header");
    };
    let content_length = content_length
        .to_str()
        .context("Content-Length header contained invalid UTF8")?;
    let content_length: u64 = content_length
        .parse()
        .context("Content-Length was not a valid 64-bit unsigned integer")?;

    Ok(content_length)
}

async fn get_range(
    http_client: &reqwest::Client,
    url: &str,
    range_start: u64,
    range_end: u64,
) -> io::Result<Bytes> {
    let range_header = format!("bytes={}-{}", range_start, range_end);
    let response = http_client
        .get(url)
        .header(RANGE, range_header)
        .send()
        .await
        .context("GET request failed")
        .map_err(io::Error::other)?;

    response
        .error_for_status_ref()
        .context("GET request returned non-success status code")
        .map_err(io::Error::other)?;

    if response.status() != StatusCode::PARTIAL_CONTENT {
        return Err(io::Error::other(anyhow::anyhow!(
            "Response to range request has an unexpected status code (expected {}, found {})",
            StatusCode::PARTIAL_CONTENT,
            response.status()
        )));
    }

    response.bytes().await.map_err(io::Error::other)
}

pub async fn get_in_parallel(
    http_client: &reqwest::Client,
    url: &str,
    total_size: u64,
    chunk_size: u64,
) -> anyhow::Result<impl Stream<Item = io::Result<Bytes>>> {
    let chunk_count = (total_size + chunk_size - 1) / chunk_size;

    let semaphore = Arc::new(Semaphore::new(CONCURRENT_REQUEST_LIMIT));
    let (tx, rx) = mpsc::channel(CONCURRENT_REQUEST_LIMIT);

    // Spawn a task to manage downloads
    tokio::spawn({
        let http_client = http_client.clone();
        let url = url.to_string();
        async move {
            for chunk_index in 0..chunk_count {
                let chunk_semaphore = semaphore.clone();
                let chunk_http_client = http_client.clone();
                let chunk_url = url.clone();
                let chunk_tx = tx.clone();

                tokio::spawn(async move {
                    let _permit = chunk_semaphore.acquire().await.unwrap();

                    let range_start = chunk_index * chunk_size;
                    let range_end = (range_start + chunk_size - 1).min(total_size - 1);

                    match get_range(&chunk_http_client, &chunk_url, range_start, range_end).await {
                        Ok(chunk) => {
                            if chunk_tx.send((chunk_index, Ok(chunk))).await.is_err() {
                                eprintln!("Failed to send chunk {}", chunk_index);
                            }
                        }
                        Err(e) => {
                            eprintln!("Error streaming chunk {}: {:?}", chunk_index, e);
                            if chunk_tx.send((chunk_index, Err(e))).await.is_err() {
                                eprintln!("Failed to send error for chunk {}", chunk_index);
                            }
                        }
                    }
                });
            }
        }
    });

    // Create a stream from the receiver
    let stream = ReceiverStream::new(rx);

    let mut buffer = BTreeMap::new();
    let mut next_offset = 0;

    let s = stream.flat_map(move |(offset, chunk)| {
        buffer.insert(offset, chunk);
        let mut bytes_to_yield = Vec::new();
        while let Some((&offset, _chunk_result)) = buffer.first_key_value() {
            if offset != next_offset {
                break;
            }
            if let Some(chunk_result) = buffer.remove(&offset) {
                match &chunk_result {
                    Ok(_) => next_offset += 1,
                    Err(_) => next_offset += 1, // Move past error
                }
                bytes_to_yield.push(chunk_result);
            }
        }

        iter(bytes_to_yield)
    });
    Ok(s)
}

pub async fn parrarel_stream(url: &str) -> anyhow::Result<impl Stream<Item = io::Result<Bytes>>> {
    let client = Client::new();
    let total_size = get_content_length(&client, url).await?;
    let chunk_size = total_size / 100;
    get_in_parallel(&client, url, total_size, chunk_size).await
}
