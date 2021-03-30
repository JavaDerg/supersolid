use crate::config::VarStack;
use crate::fatal;
use html5ever::{local_name, namespace_url, ns, Attribute, QualName};
use markup5ever_rcdom::{Handle, Node, NodeData};
use std::cell::{Cell, RefCell};
use std::collections::HashMap;
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

    fn process(&mut self, path: &Path) -> Handle;
}

#[derive(Clone)]
pub struct ProcessorConfig<'a> {
    pub(crate) out_path: &'a Path,
    pub(crate) vars: VarStack,
}

pub struct HtmlProcessor<'a> {
    pub cfg: ProcessorConfig<'a>,
    pub stack: Vec<String>,
    pub content: Vec<Vec<Handle>>,
}

pub struct MarkdownProcessor<'a> {
    pub cfg: ProcessorConfig<'a>,
    pub template: String,
}

impl<'a> Processor for HtmlProcessor<'a> {
    fn process(&mut self, path: &Path) -> Handle {
        self.stack.push(path.to_string_lossy().to_string());
        let inner = self.process_inner(path);
        self.stack.pop();
        inner
    }
}

impl<'a> HtmlProcessor<'a> {
    fn process_inner(&mut self, path: &Path) -> Handle {
        let mut handle = Self::read_handle(path);
        if let NodeData::Document = &handle.data {
            if handle.children.borrow().len() == 1 {
                let children = handle.children.borrow();
                let handle = children.first().unwrap();
                if let NodeData::Element { name, attrs, .. } = &handle.data {
                    if name.local == *"super:wrap" {
                        let src = attrs
                            .take()
                            .into_iter()
                            .find(|attr| attr.name.local == *"src");
                        if src.is_none() {
                            fatal!(
                                "Invalid warp element. No src; path={}",
                                path.to_string_lossy()
                            );
                        }
                        let src = src.unwrap();
                        self.content.push(handle.children.take());
                        return self.process(Path::new(&src.value.to_string()));
                    }
                }
            }
        }
        self.traverse(handle.clone(), &mut handle)
    }

    fn traverse(&mut self, handle: Handle, root: &mut Handle) -> Handle {
        let new = Vec::with_capacity(handle.children.borrow().len());
        let children = handle.children.replace(new);
        for el in children.into_iter() {
            if let NodeData::Element { name, attrs, .. } = &el.data {
                if let Some(name) = name.local.to_string().strip_prefix("super:") {
                    match name {
                        "wrap" => {
                            tracing::error!(
                                "Found super:wrap inside document, must always be root!; path={}",
                                self.stack.last().unwrap()
                            );
                            continue;
                        }
                        "content" => {
                            let children = self.content.pop();
                            if children.is_none() {
                                tracing::error!("Content tag found, but no content available");
                                continue;
                            }
                            let mut ch_mut = handle.children.borrow_mut();
                            for child in children.unwrap() {
                                ch_mut.push(child);
                            }
                        }
                        tag => tracing::warn!(
                            "Unknown (or unimplemented) super tag found; tag={}",
                            tag
                        ),
                    }
                }
            }
            handle.children.borrow_mut().push(el.clone());
            self.traverse(el, root);
        }
        handle
    }

    fn read_handle(path: &Path) -> Handle {
        let read = read_file(path);
        if read.trim_start().starts_with("<!DOCTYPE") {
            crate::parser::parse_document(&read)
        } else {
            let node = Node {
                data: NodeData::Document,
                parent: Default::default(),
                children: Default::default(),
            };
            *node.children.borrow_mut() = crate::parser::parse_snippet(&read);
            Handle::new(node)
        }
    }
}

impl<'a> Processor for MarkdownProcessor<'a> {
    fn process(&mut self, path: &Path) -> Handle {
        // TODO: handle template
        let src = read_file(path);
        let mut new_src = String::with_capacity(src.len());
        let mut vars = HashMap::new();
        for line in src.lines() {
            if line.trim_start().starts_with(";") {
                let pre_trimmed = line.trim_start();
                let splitter = pre_trimmed.find(':');
                if let Some(splitter) = splitter {
                    vars.insert(
                        (&pre_trimmed[1..splitter]).trim().to_string(),
                        (&pre_trimmed[splitter + 1..]).trim().to_string(),
                    );
                }
            } else {
                new_src.push_str(line);
                new_src.push('\n');
            }
        }
        let new_vars = self.cfg.vars.combine(vars);
        let mut new_cfg = self.cfg.clone();
        new_cfg.vars = new_vars;
        let markdown = crate::parser::parse_markdown(&src);

        Handle::new(Node {
            parent: Cell::new(None),
            children: RefCell::new(markdown),
            data: NodeData::Element {
                name: QualName {
                    prefix: None,
                    ns: ns!(html),
                    local: string_cache::Atom::from("super:wrap"),
                },
                attrs: RefCell::new(vec![Attribute {
                    name: QualName {
                        prefix: None,
                        ns: ns!(),
                        local: local_name!("src"),
                    },
                    value: Default::default(),
                }]),
                template_contents: None,
                mathml_annotation_xml_integration_point: false,
            },
        })
    }
}

fn read_file(path: &Path) -> String {
    match std::fs::read_to_string(path) {
        Ok(src) => src,
        Err(err) => fatal!(
            "Failed to read file; path={}; error={}",
            path.to_string_lossy(),
            err,
        ),
    }
}
