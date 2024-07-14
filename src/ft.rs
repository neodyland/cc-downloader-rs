use fasttext::FastText;
use futures_util::StreamExt;
use reqwest::get;
use std::fs::{remove_file, File};
use std::io::BufWriter;
use std::{io::Write, path::Path};

pub struct LanguagePredictor {
    ft: FastText,
}

impl LanguagePredictor {
    fn new(ft: FastText) -> Self {
        Self { ft }
    }
    pub fn predict(&self, s: &str, lab: &str) -> anyhow::Result<bool> {
        if lab == "ja" {
            for x in 'あ'..'ん' {
                if s.contains(x) {
                    return Ok(true);
                }
            }
            for x in 'ア'..'ン' {
                if s.contains(x) {
                    return Ok(true);
                }
            }
        }
        let lab = format!("__label__{}", lab);
        match self.ft.predict(s, 1, 0.0001) {
            Ok(p) => {
                for x in p {
                    if x.label == lab && x.prob > 0.6 {
                        return Ok(true);
                    }
                }
                return Ok(false);
            }
            Err(e) => anyhow::bail!(e),
        }
    }
}

pub async fn get_model() -> anyhow::Result<LanguagePredictor> {
    let pth = Path::new("./assets/fasttext.bin");
    if !pth.exists() {
        let mut pth_file = BufWriter::new(File::create(pth)?);
        let res = get("https://dl.fbaipublicfiles.com/fasttext/supervised-models/lid.176.bin")
            .await?
            .error_for_status()?;
        let mut bs = res.bytes_stream();
        while let Some(b) = bs.next().await {
            match b {
                Ok(b) => Ok(b),
                Err(e) => Err(anyhow::anyhow!(e)),
            }
            .and_then(|b| anyhow::Ok(pth_file.write(&b)?))
            .or_else(|e| {
                remove_file(pth)?;
                anyhow::bail!(e)
            })?;
        }
    }
    let mut ft = FastText::new();
    match ft.load_model("./assets/fasttext.bin") {
        Ok(_) => Ok(LanguagePredictor::new(ft)),
        Err(e) => anyhow::bail!(e),
    }
}
