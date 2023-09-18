use crate::{Function, FunctionSet};
use anyhow::anyhow;
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;
use wd_tools::{PFErr, PFOk};

#[derive(Default, Debug)]
pub struct Env {
    envs: HashMap<String, String>,
}
impl Env {
    #[allow(dead_code)]
    pub fn add_env_variable<S: Into<String>>(&mut self, key: S, value: S) {
        self.envs.insert(key.into(), value.into());
    }
    #[allow(dead_code)]
    pub fn add_env_variables<Map: Into<HashMap<String, String>>>(&mut self, map: Map) {
        let map = map.into();
        for (k, v) in map.into_iter() {
            self.add_env_variable(k, v);
        }
    }
}
impl From<HashMap<String, String>> for Env {
    fn from(envs: HashMap<String, String>) -> Self {
        Env { envs }
    }
}
impl Function for Env {
    fn call(&self, _fs: Arc<dyn FunctionSet>, args: Vec<Value>) -> anyhow::Result<Value> {
        if args.len() < 1 {
            return anyhow!("must have a args").err();
        }
        if let Value::String(key) = args.get(0).unwrap() {
            if let Some(s) = self.envs.get(key) {
                return Value::String(s.to_string()).ok();
            }
        }
        return Value::String(String::new()).ok();
    }
}
