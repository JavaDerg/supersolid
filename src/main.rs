use std::path::{Path, PathBuf};

use crate::config::Source;
use crate::processor::{HtmlProcessor, MarkdownProcessor, Processor, ProcessorConfig};
use crate::writer::Enqueuer;
use markup5ever_rcdom::SerializableHandle;
use path_clean::PathClean;
use tracing::{trace, warn};

mod config;
mod parser;
mod processor;
mod writer;

#[macro_export]
macro_rules! fatal {
    ($($token:tt)*) => {{
        tracing::error!($($token)*);
        ::std::process::exit(1);
    }};
}

fn main() {
    tracing_subscriber::fmt::init();

    if let Some(dir) = std::env::args().nth(1) {
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

    let (handle, writer) = writer::Writer::new(dist);

    let var_stack = config.get_stack(); // TODO
    for (output, src) in config.src.into_iter() {
        let cfg = processor::ProcessorConfig {
            out_path: Path::new(&output),
            vars: var_stack.clone(),
            writer: writer.clone(),
        };
        match src {
            Source::Html(src) => process(
                src,
                &output,
                writer.clone(),
                HtmlProcessor {
                    cfg,
                    stack: vec![],
                    content: vec![],
                },
            ),
            Source::Md { src, template } => process(
                src,
                &output,
                writer.clone(),
                MarkdownProcessor { cfg, template },
            ),
            Source::Copy(src) => {
                let glob = match glob::glob(&src) {
                    Ok(glob) => glob,
                    Err(err) => fatal!("Unable to glob files; path={}; error={}", src, err),
                };
                glob.map(|path| match path {
                    Ok(path) => path,
                    Err(err) => fatal!(
                        "Unable to obtain path from glob; path={}; error={}",
                        src,
                        err
                    ),
                })
                .map(|path| match path.file_name() {
                    Some(name) => (path.to_path_buf(), Path::new(&output).join(Path::new(name))),
                    None => fatal!(
                        "Path contains no file name; path={}",
                        path.to_string_lossy()
                    ),
                })
                .for_each(|(from, to)| writer.copy(from, to));
            }
            Source::For(src) => fatal!("'For' not implemented yet; src={}", src), // TODO
        }
    }

    drop(writer);
    handle.join();
}

fn process<P: Processor>(src: String, output: &str, writer: Enqueuer, mut p: P) {
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
    let out_files = match files.len() {
        0 => {
            warn!("No files found, skipping; path={}", src);
            return;
        }
        1 => vec![output.to_string()],
        _ => files
            .iter()
            .map(|path| {
                Path::new(output)
                    .join(match path.file_name() {
                        Some(name) => PathBuf::from(name),
                        None => fatal!(
                            "Path contains no file name; path={}",
                            path.to_string_lossy()
                        ),
                    })
                    .to_string_lossy()
                    .to_string()
            })
            .map(|mut file| {
                if file.ends_with(".md") {
                    file = format!("{}.html", &file[..file.len() - 3]);
                }
                file
            })
            .collect(),
    };
    let out = match files.len() {
        1 => vec![p.one(files.pop().unwrap())],
        _ => p.many(files),
    };

    let opts = html5ever::serialize::SerializeOpts {
        create_missing_parent: true,
        ..Default::default()
    };
    for (handle, output) in out
        .into_iter()
        .zip(out_files.into_iter().map(PathBuf::from))
    {
        let mut ser = Vec::new();
        html5ever::serialize(
            &mut ser,
            &Into::<SerializableHandle>::into(handle),
            opts.clone(),
        )
        .unwrap();
        writer.file(output, String::from_utf8(ser).unwrap());
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
