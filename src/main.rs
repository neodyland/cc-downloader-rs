mod cc_stream;
mod ft;
mod gz_dec;
mod warc;

use async_compression::{tokio::write::ZstdEncoder, Level};
use json::{object, stringify};
use std::time::{SystemTime, UNIX_EPOCH};

use tokio::{
    fs::File,
    io::{AsyncWriteExt, BufWriter},
};

#[tokio::main]
async fn main() {
    if let Err(e) = main_inner().await {
        eprintln!("{:?}", e);
    }
}

async fn main_inner() -> anyhow::Result<()> {
    let all = &include_str!("../paths").split("\n").collect::<Vec<_>>();
    let f = File::create(format!(
        "output/{}.zstd",
        SystemTime::now().duration_since(UNIX_EPOCH)?.as_micros()
    ))
    .await?;
    let mut f = ZstdEncoder::with_quality(BufWriter::new(f), Level::Best);
    let mut stream = cc_stream::stream_lot(&all, "ja").await;
    while let Some(s) = stream.recv().await {
        f.write(
            stringify(object! {
                html: s,
            })
            .as_bytes(),
        )
        .await?;
        f.write("\n".as_bytes()).await?;
        f.flush().await?;
    }
    Ok(())
}
