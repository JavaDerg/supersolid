use crate::fatal;
use std::collections::HashMap;
use std::path::PathBuf;
use std::rc::Rc;

#[derive(serde::Deserialize, Debug)]
pub struct ProjectConfig {
    pub vars: HashMap<String, VarSource>,
    pub dist: PathBuf,
    pub src: Vec<(String, Source)>,
}

#[derive(serde::Deserialize, Debug, Clone)]
pub enum VarSource {
    Text(String),
    Env(String),
}

#[derive(serde::Deserialize, Debug)]
pub enum Source {
    Html(String),
    Copy(String),
    Md { src: String, template: String },
    For(String),
}

#[derive(Default, Clone)]
pub struct VarStack(Rc<InnerVs>);

#[derive(Default)]
struct InnerVs {
    vars: HashMap<String, String>,
    prev: Option<Box<VarStack>>,
}

impl ProjectConfig {
    pub fn get_stack(&self) -> VarStack {
        VarStack(Rc::new(InnerVs {
            vars: self
                .vars
                .clone()
                .into_iter()
                .map(|(k, v)| {
                    (
                        k,
                        match v {
                            VarSource::Text(str) => str,
                            VarSource::Env(env) => match std::env::var(&env) {
                                Ok(str) => str,
                                Err(err) => fatal!(
                                    "Unable to obtain environment variable; name={}; error={}",
                                    env,
                                    err
                                ),
                            },
                        },
                    )
                })
                .collect::<HashMap<_, _>>(),
            prev: None,
        }))
    }
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
