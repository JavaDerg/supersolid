use std::collections::HashMap;
use std::path::{Path, PathBuf};

use crate::config::Source;
use crate::processor::{HtmlProcessor, MarkdownProcessor, Processor};
use glob::{Paths, PatternError};
use path_clean::PathClean;
use std::io::Error;
use std::process::exit;
use tracing::{error, info, trace, warn};

mod config;
mod parser;
mod processor;

#[macro_export]
macro_rules! fatal {
    ($($token:tt)*) => {{
        tracing::error!($($token)*);
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
        fatal!(
            "No config file found; path={}",
            config_path.to_string_lossy()
        );
    }

    let config = match std::fs::read_to_string(config_path)
        .map(|str| ron::from_str::<config::ProjectConfig>(&str))
    {
        Ok(Ok(config)) => config,
        Ok(Err(err)) => fatal!("Unable to parse config; error={}", err),
        Err(err) => fatal!("Unable to read config; error={}", err),
    };

    let dist = match absolute_path(&config.dist) {
        Ok(path) => path,
        Err(err) => fatal!(
            "Invalid dist path; path={}; error={}",
            config.dist.to_string_lossy(),
            err
        ),
    };

    info!("Outputting into {}", dist.to_string_lossy());
    if dist.exists() {
        warn!("Deleting old dist");
        std::fs::remove_dir_all(&dist);
    }
    trace!("Creating new dist dir; path={}", &dist.to_string_lossy());
    std::fs::create_dir_all(&dist);

    let var_stack = config.get_stack(); // TODO
    for (output, src) in config.src.into_iter() {
        let cfg = processor::ProcessorConfig {
            out_path: &dist.join(Path::new(&output)),
            vars: Default::default(),
        };
        match src {
            Source::Html(src) => process(
                src,
                HtmlProcessor {
                    cfg,
                    stack: vec![],
                    content: vec![],
                },
            ),
            Source::Md { src, template } => process(src, MarkdownProcessor { cfg, template }),
            Source::For(src) => fatal!("'For' not implemented yet; src={}", src), // TODO
        }
    }
}

fn process<P: Processor>(src: String, mut p: P) {
    let glob = match glob::glob(&src) {
        Ok(glob) => glob,
        Err(err) => fatal!("Unable to glob files; path={}; error={}", src, err),
    };
    let mut files = glob
        .map(|path| match path {
            Ok(path) => path,
            Err(err) => fatal!(
                "Unable to obtain path from glob; path={}; error={}",
                src,
                err
            ),
        })
        .collect::<Vec<_>>();
    match files.len() {
        0 => warn!("No files found, skipping; path={}", src),
        1 => p.one(files.pop().unwrap()),
        _ => p.many(files),
    }
}

// Taken from https://stackoverflow.com/a/54817755
fn absolute_path(path: impl AsRef<Path>) -> std::io::Result<PathBuf> {
    let path = path.as_ref();

    let absolute_path = if path.is_absolute() {
        path.to_path_buf()
    } else {
        std::env::current_dir()?.join(path)
    }
    .clean();

    Ok(absolute_path)
}
