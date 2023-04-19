use tokio::sync::mpsc;

pub enum GlobalHttpCommand {
    Bind {
        to: String,
        chan: mpsc::UnboundedSender<HttpMasterCommand>,
    },
}

pub enum HttpMasterCommand {}
