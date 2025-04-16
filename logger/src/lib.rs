use std::fs::File;

use colored::Colorize;
use middleware::logger::LoggerMiddleware;

pub mod middleware {
    pub mod logger;
}

pub fn setup() -> Result<(), fern::InitError> {
    File::create("snipper.log").map_err(fern::InitError::Io)?;

    fern::Dispatch::new()
        .format(|out, message, record| {
            let color = match record.level() {
                log::Level::Info => "green",
                log::Level::Warn => "yellow",
                log::Level::Error => "red",
                log::Level::Debug => "magenta",
                log::Level::Trace => "bright black",
            };
            out.finish(format_args!(
                "{}[{}][{}] {}",
                chrono::Local::now().format("[%H:%M:%S]"),
                record.target(),
                record.level().to_string().color(color),
                message
            ))
        })
        .level(log::LevelFilter::Debug)
        .level_for("ethers_providers", log::LevelFilter::Off)
        .level_for("hyper", log::LevelFilter::Off)
        .chain(std::io::stdout())
        .chain(fern::log_file("snipper.log")?)
        .apply()?;
    Ok(())
}

pub fn middleware() -> LoggerMiddleware {
    LoggerMiddleware::new()
}
