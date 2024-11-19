mod cc_stream;
mod gz_dec;
mod text;
mod warc;

use arrow::{
    array::{ArrayBuilder, RecordBatch, StringBuilder},
    datatypes::{DataType, Field, Schema},
};
use parquet::arrow::AsyncArrowWriter;
use parquet::basic::Compression;
use parquet::basic::ZstdLevel;
use parquet::file::properties::WriterProperties;
use std::{sync::Arc, time::Duration};

use tokio::{
    fs::{read_to_string, File},
    time::sleep,
};

async fn write_chunk_to_parquet(
    mut string_builder: StringBuilder,
    file_counter: usize,
) -> anyhow::Result<()> {
    let schema = Schema::new(vec![Field::new("text", DataType::Utf8, false)]);

    let batch = RecordBatch::try_new(
        Arc::new(schema.clone()),
        vec![Arc::new(string_builder.finish())],
    )?;
    let file = File::create(format!("output/part_{:05}.parquet", file_counter)).await?;

    let props = WriterProperties::builder()
        .set_compression(Compression::ZSTD(ZstdLevel::try_new(3).unwrap()))
        .build();

    let mut writer = AsyncArrowWriter::try_new(file, Arc::new(schema), Some(props))?;

    writer.write(&batch).await?;
    writer.close().await?;

    Ok(())
}

#[tokio::main]
async fn main() {
    if let Err(e) = main_inner().await {
        eprintln!("{:?}", e);
    }
}

async fn main_inner() -> anyhow::Result<()> {
    let all = read_to_string("./paths").await?.trim().to_string();
    let all = all.split('\n').collect::<Vec<_>>();
    let mut string_builder = StringBuilder::new();
    let mut file_counter = 0;
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
            string_builder.append_value(s);
            if string_builder.len() == 100_000 {
                write_chunk_to_parquet(string_builder, file_counter).await?;
                string_builder = StringBuilder::new();
                file_counter += 1;
            }
        }
        println!("Success: {a}");
    }
    if !string_builder.is_empty() {
        write_chunk_to_parquet(string_builder, file_counter).await?;
    }
    Ok(())
}
