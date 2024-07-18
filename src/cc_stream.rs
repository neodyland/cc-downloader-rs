use futures_util::StreamExt;
use reqwest::get;
use std::io;
use tokio::sync::mpsc;
use tokio_util::io::StreamReader;
use unicode_normalization::UnicodeNormalization;

use crate::{
    ft::{get_model, LanguagePredictor},
    gz_dec::GzipCmdParser,
    warc::WarcParser,
};

pub fn detect_language(ft: &LanguagePredictor, body: &str) -> bool {
    let mut is_lang = false;
    for x in 'あ'..'ん' {
        if body.contains(x) {
            is_lang = true;
        }
    }
    for x in 'ア'..'ン' {
        if body.contains(x) {
            is_lang = true;
        }
    }
    is_lang && ft.predict(body, "ja").unwrap_or(false)
}

pub async fn stream(path: &str) -> anyhow::Result<mpsc::Receiver<String>> {
    let res = get(&format!("https://data.commoncrawl.org/{path}"))
        .await?
        .error_for_status()?
        .bytes_stream()
        .map(|s| match s {
            Ok(o) => Ok(o),
            Err(e) => Err(io::Error::other(e)),
        });
    let ft = get_model().await?;
    let (send, recv) = mpsc::channel(10000);
    tokio::spawn(async move {
        let res = GzipCmdParser::new(StreamReader::new(res))?;
        let mut res = WarcParser::new(StreamReader::new(res));
        let send = send;
        while let Some(rec) = res.next().await {
            let rec = match rec {
                Ok(rec) => rec,
                Err(e) => {
                    println!("{e}");
                    continue;
                }
            };
            if let Ok(body) = String::from_utf8(rec.content) {
                if detect_language(&ft, &body) {
                    for body in crate::text::extract(&ft, &body.nfkc().to_string()) {
                        send.send(body).await?;
                    }
                }
            }
        }
        anyhow::Ok(())
    });
    Ok(recv)
}
