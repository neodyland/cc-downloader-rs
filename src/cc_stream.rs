use futures_util::StreamExt;
use htmd::{Element, HtmlToMarkdown};
use regex::Regex;
use reqwest::get;
use std::io;
use tokio::sync::mpsc;
use tokio_util::io::StreamReader;
use unicode_normalization::UnicodeNormalization;

use crate::{gz_dec::GzipCmdParser, text::detect_language, warc::WarcParser};

pub async fn stream(path: &str) -> anyhow::Result<mpsc::Receiver<String>> {
    let res = get(&format!("https://data.commoncrawl.org/{path}"))
        .await?
        .error_for_status()?
        .bytes_stream()
        .map(|s| match s {
            Ok(o) => Ok(o),
            Err(e) => Err(io::Error::other(e)),
        });
    let (send, recv) = mpsc::channel(10000);
    tokio::spawn(async move {
        let res = GzipCmdParser::new(StreamReader::new(res))?;
        let mut res = WarcParser::new(StreamReader::new(res));
        let send = send;
        let converter = HtmlToMarkdown::builder()
            .skip_tags(vec!["script", "style", "img", "video"])
            .add_handler(vec!["a"], |e: Element| Some(e.content.to_string()))
            .build();
        let re_jp = Regex::new(
            r"([ぁ-んァ-ン -~！”＃＄％＆’（）*+，−．／：；＜＝＞？＠［＼］＾＿｀｛｜｝〜]+)",
        )
        .unwrap();
        while let Some(rec) = res.next().await {
            let rec = match rec {
                Ok(rec) => rec,
                Err(_e) => {
                    continue;
                }
            };
            if detect_language(&rec.content) {
                if let Ok(body) =
                    crate::text::extract(&converter, &re_jp, &rec.content.nfkc().to_string())
                {
                    send.send(body).await?;
                }
            }
        }
        anyhow::Ok(())
    });
    Ok(recv)
}
