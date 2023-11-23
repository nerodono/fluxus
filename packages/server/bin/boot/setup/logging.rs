use color_eyre::eyre::{
    self,
    Context,
};
use fluxus::config::logging::{
    LogLevel,
    LoggingConfig,
};
use tracing::Level;
use tracing_subscriber::FmtSubscriber;

fn convert_config_level(level: LogLevel) -> Option<Level> {
    use LogLevel as L;
    Some(match level {
        L::Info => Level::INFO,
        L::Error => Level::ERROR,
        L::Debug => Level::DEBUG,
        L::Disabled => return None,
    })
}

pub fn install_tracing(config: &LoggingConfig) -> eyre::Result<()> {
    let Some(level) = convert_config_level(config.level) else {
        // Logging is turned off
        return Ok(());
    };

    let sub = FmtSubscriber::builder()
        .with_max_level(level)
        .without_time()
        .compact()
        .finish();
    tracing::subscriber::set_global_default(sub)
        .wrap_err("failed to set up global subscriber")?;

    Ok(())
}
