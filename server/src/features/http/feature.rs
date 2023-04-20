use tokio::sync::mpsc;

use crate::{
    data::commands::http::GlobalHttpCommand,
    error::{
        NonCriticalError,
        NonCriticalResult,
    },
};

#[derive(Clone)]
pub struct HttpFeature {
    chan: mpsc::UnboundedSender<GlobalHttpCommand>,
}

impl HttpFeature {
    pub fn send_command(
        &self,
        command: GlobalHttpCommand,
    ) -> NonCriticalResult<()> {
        self.chan
            .send(command)
            .map_err(|_| NonCriticalError::FeatureIsDisabled)
    }

    pub const fn new(chan: mpsc::UnboundedSender<GlobalHttpCommand>) -> Self {
        Self { chan }
    }
}
