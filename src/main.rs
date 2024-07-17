mod cc_stream;
mod ft;
mod gz_dec;
mod html2md;
mod warc;

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
    let mut f = BufWriter::new(
        File::create(format!(
            "output/{}.jsonl",
            SystemTime::now().duration_since(UNIX_EPOCH)?.as_micros()
        ))
        .await?,
    );
    for a in all {
        if let Ok(mut stream) = cc_stream::stream(a, "ja").await {
            while let Some(s) = stream.recv().await {
                f.write_all(
                    stringify(object! {
                        html: s,
                    })
                    .as_bytes(),
                )
                .await?;
                f.write_all("\n".as_bytes()).await?;
            }
            println!("Success: {a}");
        } else {
            println!("Failed: {a}");
        }
    }
    f.shutdown().await?;
    Ok(())
}
