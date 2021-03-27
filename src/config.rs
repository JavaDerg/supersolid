use std::collections::HashMap;
use std::rc::Rc;
use std::path::{PathBuf, Path};

#[derive(serde::Deserialize)]
pub struct ProjectConfig {
    pub vars: HashMap<String, VarSource>,
    pub dist: PathBuf,
    pub src: HashMap<String, Source>
}

#[derive(serde::Deserialize)]
pub enum VarSource {
    Text(String),
    Env(String),
}

#[derive(serde::Deserialize)]
pub enum Source {
    Html(String),
    Md {
        src: String,
        template: String,
    },
    For(String),
}

#[derive(Default, Clone)]
pub struct VarStack(Rc<InnerVs>);

#[derive(Default)]
struct InnerVs {
    vars: HashMap<String, String>,
    prev: Option<Box<VarStack>>,
}

impl VarStack {
    pub fn get(&self, key: &str) -> Option<String> {
        match self.0.vars.get(key).cloned() {
            Some(str) => Some(str),
            None => {
                if let Some(prev) = &self.0.prev {
                    prev.get(key)
                } else {
                    None
                }
            }
        }
    }

    pub fn combine(&self, vars: HashMap<String, String>) -> VarStack {
        VarStack(Rc::new(InnerVs {
            vars,
            prev: Some(Box::new(self.clone())),
        }))
    }
}
