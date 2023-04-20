use crate::{
    data::commands::http::{
        GlobalHttpCommand,
        HttpMasterCommand,
    },
    features::http::storage::HttpStorage,
};

pub async fn handle_command(
    command: GlobalHttpCommand,
    storage: &HttpStorage,
) {
    match command {
        GlobalHttpCommand::Bind { to, permit, pool } => {
            // TODO: Random endpoint selection
            let endpoint = to.unwrap();
            if let Err(error) = storage
                .try_bind_endpoint(endpoint, pool, &permit)
                .await
            {
                _ = permit
                    .send(HttpMasterCommand::FailedToBindEndpoint { error });
                return;
            }

            // TODO: Random endpoint selection
            _ = permit.send(HttpMasterCommand::BoundEndpoint { on: None });
        }

        GlobalHttpCommand::Unbind { domain_or_path } => {
            storage.unbind_endpoint(&domain_or_path).await;
        }
    }
}
