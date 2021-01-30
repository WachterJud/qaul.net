//! A subscriber-log wrapper
//!
//! The subscriber-fmt crate has a bug that doesn't properly handle
//! ENV variables to change log levels.  This module fixes this.

use tracing::info;
use tracing_subscriber::{filter::LevelFilter, fmt, EnvFilter};

pub(crate) fn parse_log_level() {
    let filter = EnvFilter::try_from_env("QAUL_LOG")
        .unwrap_or_default()
        .add_directive(LevelFilter::TRACE.into())
        .add_directive("async_std=error".parse().unwrap())
        .add_directive("mio=error".parse().unwrap());

    // Initialise the logger
    fmt().with_env_filter(filter).init();
    info!("Initialised logger: welcome to qaul-hubd!");
}
