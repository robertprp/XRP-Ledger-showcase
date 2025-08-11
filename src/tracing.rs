use tracing::{Level, level_filters::LevelFilter};
use tracing_subscriber::{
    filter::Targets, fmt::writer::MakeWriterExt, layer::SubscriberExt, util::SubscriberInitExt,
};

pub fn init() -> Result<(), String> {
    let log_level = LevelFilter::INFO;

    let stdout_log_layer = tracing_subscriber::fmt::layer()
        .compact()
        .with_ansi(true)
        .with_target(true)
        .with_writer(std::io::stdout.with_max_level(Level::INFO));

    let target = Targets::new().with_target("shogun_xrp", log_level);

    let registry = tracing_subscriber::Registry::default()
        .with(target)
        .with(stdout_log_layer);

    // Initialize global subscriber
    registry.init();

    Ok(())
}
