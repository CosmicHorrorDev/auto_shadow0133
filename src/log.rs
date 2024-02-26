use std::io;

use tracing_log::LogTracer;
use tracing_subscriber::{filter::LevelFilter, EnvFilter};

pub fn init() -> anyhow::Result<()> {
    let subscriber = tracing_subscriber::fmt()
        .compact()
        .with_writer(io::stderr)
        .with_env_filter(
            EnvFilter::builder()
                .with_default_directive(LevelFilter::INFO.into())
                .with_env_var("LOG")
                .from_env()?,
        )
        .finish();
    tracing::subscriber::set_global_default(subscriber)?;
    LogTracer::init()?;

    Ok(())
}
