use futures_util::StreamExt;
use reqwest::get;
use std::collections::HashMap;
use tl::{parse, ParserOptions};
use tokio::sync::mpsc;
use tokio_util::io::StreamReader;
use unicode_normalization::UnicodeNormalization;

use crate::{
    ft::{get_model, LanguagePredictor},
    gz_dec::GzipCmdParser,
    warc::WarcParser,
};

static METAS: [&str; 18] = [
    "dc.description",
    "dc:description",
    "dcterms.abstract",
    "dcterms.description",
    "description",
    "sailthru.description",
    "twitter:description",
    "citation_title",
    "dc.title",
    "dcterms.title",
    "fb_title",
    "headline",
    "parsely-title",
    "sailthru.title",
    "shareaholic:title",
    "rbtitle",
    "title",
    "twitter:title",
];

fn detect_language(ft: &LanguagePredictor, body: &str, lang: &str) -> Option<String> {
    let parsed = parse(body, ParserOptions::new()).ok()?;
    let mut is_lang = false;
    if let Some(meta_desc) = parsed.query_selector("head > title") {
        for e in meta_desc {
            if let Some(e) = e.get(parsed.parser()) {
                let e = e.inner_text(parsed.parser());
                if e.len() > 5 {
                    if let Ok(true) = ft.predict(&e, lang) {
                        is_lang = true;
                        break;
                    }
                }
            };
        }
    };
    if !is_lang {
        if let Some(meta_desc) = parsed.query_selector("html") {
            for e in meta_desc {
                if let Some(e) = e.get(parsed.parser()) {
                    if let Some(e) = e.as_tag() {
                        if let Some(Some(e)) = e.attributes().get("lang") {
                            let e = String::from_utf8_lossy(e.as_bytes()).to_string();
                            if e == "ja".to_string() {
                                is_lang = true;
                            }
                        }
                    }
                };
            }
        };
    }
    if !is_lang {
        if let Some(e) = parsed.query_selector("meta") {
            for e in e {
                if let Some(e) = e.get(parsed.parser()) {
                    if let Some(e) = e.as_tag() {
                        let mut ok = false;
                        if let Some(Some(e)) = e.attributes().get("name") {
                            let e = String::from_utf8_lossy(e.as_bytes()).to_string();
                            if METAS.contains(&e.as_str()) {
                                ok = true;
                            }
                        }
                        if let Some(Some(e)) = e.attributes().get("property") {
                            let e = String::from_utf8_lossy(e.as_bytes()).to_string();
                            if METAS.contains(&e.as_str()) {
                                ok = true;
                            }
                        }
                        if !ok {
                            continue;
                        }
                        if let Some(Some(e)) = e.attributes().get("content") {
                            if let Ok(true) =
                                ft.predict(&String::from_utf8_lossy(e.as_bytes()).to_string(), lang)
                            {
                                is_lang = true;
                                break;
                            }
                        }
                    }
                }
            }
        }
    }
    if !is_lang {
        'a: for t in ["h1", "h2", "h3", "h4", "h5", "h6", "p", "a"] {
            for e in parsed.get_elements_by_class_name(t) {
                if let Some(e) = e.get(parsed.parser()) {
                    let t = e.inner_text(parsed.parser());
                    if t.len() > 5 {
                        if let Ok(true) = ft.predict(&t, lang) {
                            is_lang = true;
                            break 'a;
                        }
                    }
                };
            }
        }
    }
    if !is_lang {
        return None;
    }
    Some(body.to_string())
}

fn split_headers(s: &str) -> anyhow::Result<(HashMap<String, String>, String)> {
    let s = s.replace("\r", "");
    if let Some((h, o)) = s.split_once("\n\n") {
        let mut headers = HashMap::new();
        for l in h.split("\n") {
            if let Some((n, v)) = l.split_once(":") {
                headers.insert(n.to_lowercase(), v.to_string());
            }
        }
        Ok((headers, o.to_string()))
    } else {
        anyhow::bail!("Failed")
    }
}

pub async fn stream(path: &str, lang: &str) -> anyhow::Result<mpsc::Receiver<String>> {
    let cvt = crate::html2md::get_converter();
    let res = get(format!("https://data.commoncrawl.org/{path}"))
        .await?
        .error_for_status()?;
    let ft = get_model().await?;
    let (send, recv) = mpsc::channel(10000);
    let lang = lang.to_string();
    tokio::spawn(async move {
        let res = res.bytes_stream().map(|s| match s {
            Ok(s) => Ok(s),
            Err(e) => Err(std::io::Error::other(e)),
        });
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
            if rec.header.get("warc-type") == Some(&"response".to_string()) {
                let s = String::from_utf8_lossy(&rec.content).to_string();
                let (headers, body) = match split_headers(&s) {
                    Ok(x) => x,
                    Err(_) => continue,
                };
                if let Some(ct) = headers.get("content-type") {
                    if ct.contains("text/html") {
                        if let Some(body) = detect_language(&ft, &body.nfkc().to_string(), &lang) {
                            if let Some(body) = crate::html2md::extract(&cvt, &body) {
                                if let Ok(body) = String::from_utf8(body.as_bytes().to_vec()) {
                                    send.send(body).await?;
                                } else {
                                    println!("Found garbled characters!");
                                }
                            }
                        }
                    }
                }
            }
        }
        anyhow::Ok(())
    });
    Ok(recv)
}
