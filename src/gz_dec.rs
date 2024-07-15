use futures_util::Future;
use futures_util::{stream::Stream, StreamExt};
use std::process::Stdio;
use tokio::io::AsyncWriteExt;
use tokio::sync::mpsc;
use tokio::{io::AsyncReadExt, process::Command};
use tokio_util::{bytes::Bytes, io::ReaderStream};
pub struct GzipCmdParser {
    recv: mpsc::Receiver<Bytes>,
}

impl GzipCmdParser {
    pub fn new<R: AsyncReadExt + Unpin + Send + 'static>(reader: R) -> anyhow::Result<Self> {
        let mut reader = ReaderStream::new(reader);
        let cmd = Command::new("gunzip")
            .arg("-")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()?;
        let mut stdin = cmd.stdin.unwrap();
        let mut stdout = ReaderStream::new(cmd.stdout.unwrap());
        let (send, recv) = mpsc::channel(10000);
        tokio::spawn(async move {
            while let Some(Ok(b)) = reader.next().await {
                stdin.write_all(&b).await.ok();
                stdin.flush().await.ok();
            }
        });
        tokio::spawn(async move {
            while let Some(Ok(b)) = stdout.next().await {
                send.send(b).await.ok();
            }
        });
        Ok(GzipCmdParser { recv })
    }

    pub async fn next_chunk(&mut self) -> std::io::Result<Option<Bytes>> {
        Ok(self.recv.recv().await)
    }
}

impl Stream for GzipCmdParser {
    type Item = std::io::Result<Bytes>;

    fn poll_next(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Option<Self::Item>> {
        use std::task::Poll;
        let future = self.next_chunk();
        futures_util::pin_mut!(future);
        match future.poll(cx) {
            Poll::Ready(Ok(Some(record))) => Poll::Ready(Some(Ok(record))),
            Poll::Ready(Ok(None)) => Poll::Ready(None),
            Poll::Ready(Err(e)) => Poll::Ready(Some(Err(e))),
            Poll::Pending => Poll::Pending,
        }
    }
}
