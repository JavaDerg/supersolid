use crate::config::VarStack;
use std::path::{Path, PathBuf};
#[macro_use]
use crate::fatal;
use html5ever::{Attribute, QualName};
use markup5ever_rcdom::{Handle, NodeData};
use std::cell::RefCell;
use std::io::Error;

pub trait Processor {
    fn one(&mut self, path: PathBuf) {
        self.process(&path);
    }

    fn many(&mut self, path_bufs: Vec<PathBuf>) {
        for path in path_bufs.into_iter() {
            self.process(&path);
        }
    }

    fn process(&mut self, path: &Path);
}

pub struct ProcessorConfig<'a> {
    pub(crate) out_path: &'a Path,
    pub(crate) vars: VarStack,
}

pub struct HtmlProcessor<'a> {
    pub cfg: ProcessorConfig<'a>,
    pub stack: Vec<String>,
}

pub struct MarkdownProcessor<'a> {
    pub cfg: ProcessorConfig<'a>,
    pub template: String,
}

impl<'a> Processor for HtmlProcessor<'a> {
    fn process(&mut self, path: &Path) {
        self.traverse(path, self.read_handle(path));
    }
}

impl<'a> HtmlProcessor<'a> {
    fn read_handle(&self, path: &Path) -> Handle {
        let read = read_file(path);
        if read.trim_start().starts_with("<!DOCTYPE") {
            crate::parser::parse_document(&read)
        } else {
            crate::parser::parse_snippet(&read)
        }
    }

    fn enter_document(&mut self, path: &Path, handle: Handle) -> Handle {
        self.stack.push(path.to_string_lossy().to_string());
        let handle = self.traverse(path, handle);
        self.stack.pop();
        handle
    }

    fn traverse(&mut self, path: &Path, mut handle: Handle) -> Handle {
        for el in handle.children.borrow().iter() {
            match &el.data {
                NodeData::Element { name, attrs, .. } => {
                    if name.ns == *"super"
                        || attrs.borrow().iter().any(|attr| attr.name.ns == *"super")
                    {
                        tracing::info!("{}", name.ns.to_string());
                        self.exec(&handle, name, attrs);
                    }
                }
                _ => (),
            }
            self.traverse(path, el.clone());
        }
        handle
    }

    fn exec(&mut self, handle: &Handle, name: &QualName, attrs: &RefCell<Vec<Attribute>>) {
        todo!()
    }
}

impl<'a> Processor for MarkdownProcessor<'a> {
    fn process(&mut self, path: &Path) {
        unimplemented!()
    }
}

fn read_file(path: &Path) -> String {
    match std::fs::read_to_string(path) {
        Ok(src) => src,
        Err(err) => fatal!(
            "Failed to read file; path={}; error={}",
            path.to_string_lossy(),
            err
        ),
    }
}
