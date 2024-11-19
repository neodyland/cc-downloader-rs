use futures_util::stream::Stream;
use futures_util::Future;
use std::collections::HashMap;
use tokio::io::{AsyncBufReadExt, AsyncReadExt, BufReader};

#[derive(Debug)]
pub struct WarcRecord {
    #[allow(dead_code)]
    pub header: HashMap<String, String>,
    pub content: String,
}

pub struct WarcParser<R: AsyncReadExt + Unpin> {
    reader: BufReader<R>,
}

impl<R: AsyncReadExt + Unpin> WarcParser<R> {
    pub fn new(reader: R) -> Self {
        WarcParser {
            reader: BufReader::new(reader),
        }
    }

    pub async fn next_record(&mut self) -> anyhow::Result<Option<WarcRecord>> {
        loop {
            let mut b = Vec::new();
            if self.reader.read_until(b'\n', &mut b).await? == 0 {
                return Ok(None); // End of input
            }
            let line = String::from_utf8_lossy(&b).to_string();

            if line.trim() == "WARC/1.0" {
                return Ok(Some(self.parse_record().await?));
            }
        }
    }

    async fn parse_record(&mut self) -> anyhow::Result<WarcRecord> {
        let mut header = HashMap::new();
        let mut content_length = 0;
        loop {
            let mut line = String::new();
            if self.reader.read_line(&mut line).await? == 0 {
                break;
            }

            let line = line.trim();
            if line.is_empty() {
                break;
            }

            if let Some((key, value)) = line.split_once(':') {
                let key = key.trim().to_lowercase();
                let value = value.trim().to_string();
                if key == "content-length" {
                    content_length = value.parse().unwrap_or(0);
                }
                header.insert(key, value);
            }
        }

        let mut content = vec![0; content_length];
        self.reader.read_exact(&mut content).await?;

        // Read and discard the two newlines after the content
        let mut buffer = [0; 2];
        self.reader.read_exact(&mut buffer).await?;
        let content = Self::try_decode(encoding_rs::UTF_8, &content)
            .or_else(|_| Self::try_decode(encoding_rs::SHIFT_JIS, &content))?;
        let content = crate::http::parse_http_response(&content)
            .ok_or(anyhow::anyhow!("http parse failed"))?;
        Ok(WarcRecord { header, content })
    }
    fn try_decode(
        encoding: &'static encoding_rs::Encoding,
        content: &[u8],
    ) -> anyhow::Result<String> {
        let (c, _, err) = encoding.decode(content);
        if !err {
            Ok(c.to_string())
        } else {
            anyhow::bail!("Encoding error")
        }
    }
}

impl<R: AsyncReadExt + Unpin> Stream for WarcParser<R> {
    type Item = anyhow::Result<WarcRecord>;

    fn poll_next(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Option<Self::Item>> {
        use std::task::Poll;
        let future = self.next_record();
        futures_util::pin_mut!(future);
        match future.poll(cx) {
            Poll::Ready(Ok(Some(record))) => Poll::Ready(Some(Ok(record))),
            Poll::Ready(Ok(None)) => Poll::Ready(None),
            Poll::Ready(Err(e)) => Poll::Ready(Some(Err(e))),
            Poll::Pending => Poll::Pending,
        }
    }
}
