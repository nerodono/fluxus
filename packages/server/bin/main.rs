use std::sync::Arc;

use boot::{
    runner::run_fut,
    setup::{
        configuration::load_config,
        environment::Environment,
        logging::install_tracing,
        runtime::create_runtime,
    },
};
use color_eyre::eyre;
use fluxus::{
    config::root::Config,
    proxies::queues::Queues,
};

async fn entrypoint(config: Config) -> eyre::Result<()> {
    use fluxus::protocols as prot;

    let config = Arc::new(config);
    let queues = Queues::default();
    let futures = [
        #[cfg(feature = "tcpflux")]
        run_fut(
            "tcpflux",
            prot::tcp_flux::run(queues.clone(), config.clone()),
        ),
    ];

    futures_util::future::join_all(futures).await;

    _ = config;
    Ok(())
}

fn main() -> eyre::Result<()> {
    color_eyre::install()?;

    let env = Environment::try_parse()?;
    let config = load_config(env.config_path.as_deref())?;
    install_tracing(&config.logging)?;

    let rt = create_runtime(config.runtime.threads)?;

    rt.block_on(entrypoint(config))
}

mod boot;
