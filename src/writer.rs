use std::path::{Path, PathBuf};
use std::sync::mpsc::{Receiver, Sender};
use std::thread::JoinHandle;

pub struct Writer {
    th: Option<JoinHandle<()>>,
}

struct InnerWriter {
    out_dir: PathBuf,
    recv: Receiver<Command>,
}

#[derive(Clone)]
pub struct Enqueuer(Sender<Command>);

enum Command {
    Write(PathBuf, String),
    Copy(PathBuf, PathBuf),
}

impl Writer {
    pub fn new(path: PathBuf) -> (Writer, Enqueuer) {
        let (tx, rx) = std::sync::mpsc::channel();
        let writer = InnerWriter {
            out_dir: path,
            recv: rx,
        };
        (
            Writer {
                th: Some(std::thread::spawn(move || {
                    writer.init();
                    for cmd in writer.recv.iter() {
                        match cmd {
                            Command::Write(path, data) => writer.write(&path, &data),
                            Command::Copy(from, to) => writer.copy(&from, &to),
                        }
                    }
                })),
            },
            Enqueuer(tx),
        )
    }

    pub fn join(self) {
        drop(self);
    }
}

impl InnerWriter {
    fn init(&self) {
        tracing::info!("Outputting into {}", self.out_dir.to_string_lossy());
        if self.out_dir.exists() {
            tracing::warn!("Deleting old dist");
            if let Err(err) = std::fs::remove_dir_all(&self.out_dir) {
                crate::fatal!(
                    "Failed to delete old dist dir; path={}; error={}",
                    &self.out_dir.to_string_lossy(),
                    err
                );
            }
        }
        tracing::trace!(
            "Creating new dist dir; path={}",
            self.out_dir.to_string_lossy()
        );
        if let Err(err) = std::fs::create_dir_all(&self.out_dir) {
            crate::fatal!(
                "Unable to create output dir; path={}; error={}",
                &self.out_dir.to_string_lossy(),
                err
            );
        }
    }

    fn write(&self, path: &Path, data: &str) {
        let path = self.path(path);
        tracing::trace!(
            "Writing file; path={}; len={}",
            path.to_string_lossy(),
            data.len()
        );
        if let Err(err) = std::fs::write(&path, data) {
            crate::fatal!(
                "Unable to write file; path={}; error={}",
                path.to_string_lossy(),
                err
            );
        };
    }

    fn copy(&self, from: &Path, to: &Path) {
        let to = self.path(to);
        if let Err(err) = std::fs::copy(from, &to) {
            crate::fatal!(
                "Unable to copy file into; from={}; to={}; error={}",
                from.to_string_lossy(),
                to.to_string_lossy(),
                err
            );
        }
    }

    fn path(&self, path: &Path) -> PathBuf {
        let path = self.out_dir.join(path);
        let parent = path.parent().unwrap();
        if !parent.exists() {
            if let Err(err) = std::fs::create_dir_all(parent) {
                crate::fatal!(
                    "Unable to create dir; path={}; error={}",
                    parent.to_string_lossy(),
                    err
                );
            }
        }
        path
    }
}

impl Enqueuer {
    pub fn file(&self, path: PathBuf, content: String) {
        self.0.send(Command::Write(path, content)).unwrap();
    }

    pub fn copy(&self, from: PathBuf, to: PathBuf) {
        self.0.send(Command::Copy(from, to)).unwrap();
    }

    pub fn copy_maybe(&self, from: PathBuf, to: PathBuf) {
        if !from.exists() || !from.is_file() {
            return;
        }
        self.copy(from, to);
    }
}

impl Drop for Writer {
    fn drop(&mut self) {
        if let Some(handle) = self.th.take() {
            let _ = handle.join();
        }
    }
}
