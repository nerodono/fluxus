use std::{
    mem,
    ptr,
};

use tokio::sync::oneshot;

struct Token;

pub struct ShutdownToken {
    rx: oneshot::Receiver<Token>,
}

pub struct ShutdownSender {
    tx: oneshot::Sender<Token>,
}

pub struct RaiiShutdown {
    inner: Option<ShutdownSender>,
}

pub fn shutdown_token() -> (ShutdownToken, ShutdownSender) {
    let (tx, rx) = oneshot::channel();
    (ShutdownToken { rx }, ShutdownSender { tx })
}

impl ShutdownToken {
    pub async fn wait_for_shutdown(&mut self) {
        _ = (&mut self.rx).await;
    }
}

impl ShutdownSender {
    pub fn shutdown(self) {
        _ = self.tx.send(Token);
    }
}

impl RaiiShutdown {
    pub fn into_inner(self) -> ShutdownSender {
        // SAFETY: safe since we're forgetting to drop the
        // RaiiShutdown, so Drop for `ShutdownSender` will
        // not be called in the end of scope
        let inner = unsafe {
            ptr::read(ptr::addr_of!(self.inner)).unwrap_unchecked()
        };
        mem::forget(self);
        inner
    }
}

impl Drop for RaiiShutdown {
    fn drop(&mut self) {
        unsafe { mem::take(&mut self.inner).unwrap_unchecked() }.shutdown();
    }
}

impl From<RaiiShutdown> for ShutdownSender {
    fn from(value: RaiiShutdown) -> Self {
        value.into_inner()
    }
}

impl From<ShutdownSender> for RaiiShutdown {
    fn from(value: ShutdownSender) -> Self {
        Self { inner: Some(value) }
    }
}
