use std::mem::take;

use tokio::sync::oneshot;

struct Token;

pub struct ShutdownTrigger {
    // Option is needed to consume sender on drop
    chan: Option<oneshot::Sender<Token>>,
}

pub struct ShutdownListener {
    chan: oneshot::Receiver<Token>,
}

pub fn shutdown_token() -> (ShutdownTrigger, ShutdownListener) {
    let (tx, rx) = oneshot::channel();
    (
        ShutdownTrigger { chan: Some(tx) },
        ShutdownListener { chan: rx },
    )
}

impl ShutdownListener {
    pub async fn wait_for_cancelation(&mut self) {
        let _ = (&mut self.chan).await;
    }
}

impl Drop for ShutdownTrigger {
    fn drop(&mut self) {
        let chan = take(&mut self.chan).unwrap();
        let _ = chan.send(Token);
    }
}
