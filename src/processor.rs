use crate::config::VarStack;
use std::path::{Path, PathBuf};

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
}

pub struct MarkdownProcessor<'a> {
    pub cfg: ProcessorConfig<'a>,
    pub template: String,
}

impl<'a> Processor for HtmlProcessor<'a> {
    fn process(&mut self, path: &Path) {
        unimplemented!()
    }
}

impl<'a> Processor for MarkdownProcessor<'a> {
    fn process(&mut self, path: &Path) {
        unimplemented!()
    }
}
