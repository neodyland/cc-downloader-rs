mod cc_stream;
mod ft;
mod gz_dec;
mod text;
mod warc;

use json::{object, stringify};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use tokio::{
    fs::{read_to_string, File},
    io::{AsyncWriteExt, BufWriter},
    time::sleep,
};

#[tokio::main]
async fn main() {
    if let Err(e) = main_inner().await {
        eprintln!("{:?}", e);
    }
}

async fn main_inner() -> anyhow::Result<()> {
    ft::get_model().await?;
    let all = read_to_string("./paths").await?.trim().to_string();
    let all = all.split('\n').collect::<Vec<_>>();
    let mut f = BufWriter::new(
        File::create(format!(
            "output/{}.jsonl",
            SystemTime::now().duration_since(UNIX_EPOCH)?.as_micros()
        ))
        .await?,
    );
    for a in all {
        let mut stream = cc_stream::stream(a).await;
        let mut attempt = 0;
        while stream.is_err() {
            println!("Attempt {attempt}");
            sleep(Duration::from_secs(attempt)).await;
            stream = cc_stream::stream(a).await;
            attempt += 1;
        }
        let mut stream = stream.unwrap();
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
    }
    f.shutdown().await?;
    Ok(())
}
