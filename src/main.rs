use std::collections::HashMap;
use std::path::{Path, PathBuf};

use tracing::{info, warn, error, trace};
use std::process::exit;
use path_clean::PathClean;
use std::io::Error;

mod config;
mod parser;

macro_rules! fatal {
    ($($token:tt)+) => {{
        tracing::error!($($token)+);
        ::std::process::exit(1);
    }};
}

fn main() {
    tracing_subscriber::fmt::init();

    if let Some(dir) = std::env::args().skip(1).next() {
        if let Err(err) = std::env::set_current_dir(&dir) {
            fatal!("Unable to set working directory; error={}", err);
        }
        trace!("Set working dir; directory={}", &dir);
    }

    let config_path = absolute_path(Path::new("config.ron"));
    if let Err(err) = &config_path {
        fatal!("Invalid path provided; error={}", err);
    }
    let config_path = config_path.unwrap();
    if !config_path.exists() || !config_path.is_file() {
        fatal!("No config file found; path={}", config_path.to_string_lossy());
    }

    let config = match std::fs::read_to_string(config_path).map(|str| ron::from_str::<config::ProjectConfig>(&str)) {
        Ok(Ok(config)) => config,
        Ok(Err(err)) => fatal!("Unable to parse config; error={}", err),
        Err(err) => fatal!("Unable to read config; error={}", err),
    };

    info!("Outputing into {}", config.dist.to_string_lossy())
}

// Taken from https://stackoverflow.com/a/54817755
fn absolute_path(path: impl AsRef<Path>) -> std::io::Result<PathBuf> {
    let path = path.as_ref();

    let absolute_path = if path.is_absolute() {
        path.to_path_buf()
    } else {
        std::env::current_dir()?.join(path)
    }.clean();

    Ok(absolute_path)
}
