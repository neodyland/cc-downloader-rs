mod cc_stream;
mod fast_dl;
mod ft;
mod gz_dec;
mod html2md;
mod warc;

use async_compression::{tokio::write::ZstdEncoder, Level};
use json::{object, stringify};
use std::time::{SystemTime, UNIX_EPOCH};

use tokio::{
    fs::{read_to_string, File},
    io::{AsyncWriteExt, BufWriter},
};

#[tokio::main]
async fn main() {
    if let Err(e) = main_inner().await {
        eprintln!("{:?}", e);
    }
}

async fn main_inner() -> anyhow::Result<()> {
    ft::get_model().await?;
    let all = read_to_string("./paths").await?;
    let all = all.split('\n').collect::<Vec<_>>();
    for a in all {
        if let Ok(mut stream) = cc_stream::stream(a, "ja").await {
            let f = File::create(format!(
                "output/{}.jsonl.zstd",
                SystemTime::now().duration_since(UNIX_EPOCH)?.as_micros()
            ))
            .await?;
            let mut f = ZstdEncoder::with_quality(BufWriter::new(f), Level::Best);
            while let Some(s) = stream.recv().await {
                f.write_all(
                    stringify(object! {
                        html: s,
                    })
                    .as_bytes(),
                )
                .await?;
                f.write_all("\n".as_bytes()).await?;
                f.flush().await?;
            }
            f.shutdown().await?;
            println!("Success: {a}");
        } else {
            println!("Failed: {a}");
        }
    }
    Ok(())
}
