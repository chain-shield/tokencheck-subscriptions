use colored::Colorize;
use fern::colors::{Color, ColoredLevelConfig};
use log::LevelFilter;


pub mod middleware;

/// Sets up the logger for the application.
/// This function initializes both console and file logging.
pub fn setup() -> Result<(), Box<dyn std::error::Error>> {
    let colors = ColoredLevelConfig::new()
        .error(Color::Red)
        .warn(Color::Yellow)
        .info(Color::Green)
        .debug(Color::Blue)
        .trace(Color::Magenta);

    fern::Dispatch::new()
        .format(move |out, message, record| {
            out.finish(format_args!(
                "{} {} [{}] {}",
                chrono::Local::now().format("[%Y-%m-%d %H:%M:%S]").to_string().bright_black(),
                colors.color(record.level()),
                record.target().bright_blue(),
                message
            ))
        })
        .level(LevelFilter::Info)
        .chain(std::io::stdout())
        .chain(fern::log_file("snipper.log")?)
        .apply()
        .map_err(|e| Box::new(e) as Box<dyn std::error::Error>)?;

    Ok(())
}

/// Creates a logger middleware for Actix Web.
/// This middleware logs HTTP requests and responses.
pub fn middleware(console_logging_enabled: bool) -> middleware::LoggerMiddleware {
    middleware::LoggerMiddleware::new(console_logging_enabled)
}
