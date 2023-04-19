use std::mem;

use tokio::sync::oneshot;

struct Token;

pub struct ShutdownPermit {
    chan: Option<oneshot::Sender<Token>>,
}

pub struct ShutdownToken {
    chan: oneshot::Receiver<Token>,
}

impl ShutdownToken {
    #[inline]
    pub async fn wait_for_shutdown(&mut self) {
        _ = (&mut self.chan).await;
    }
}

#[inline]
pub fn shutdown_token() -> (ShutdownToken, ShutdownPermit) {
    let (tx, rx) = oneshot::channel();
    (
        ShutdownToken { chan: rx },
        ShutdownPermit { chan: Some(tx) },
    )
}

impl Drop for ShutdownPermit {
    fn drop(&mut self) {
        // SAFETY: safe since we don't expose `chan` property, it is
        // always Some(channel)
        let chan = unsafe { mem::take(&mut self.chan).unwrap_unchecked() };
        _ = chan.send(Token);
    }
}
