use std::mem;

use tokio::sync::oneshot;

#[derive(Clone, Copy)]
struct Token;

pub struct ShutdownWriter {
    tx: Option<oneshot::Sender<Token>>,
}

pub struct ShutdownReader {
    rx: oneshot::Receiver<Token>,
}

impl ShutdownReader {
    pub async fn read(&mut self) {
        _ = (&mut self.rx).await;
    }
}

pub fn shutdown_token() -> (ShutdownReader, ShutdownWriter) {
    let (tx, rx) = oneshot::channel();
    (ShutdownReader { rx }, ShutdownWriter { tx: Some(tx) })
}

impl Drop for ShutdownWriter {
    fn drop(&mut self) {
        if let Some(tx) = mem::take(&mut self.tx) {
            _ = tx.send(Token);
        }
    }
}
