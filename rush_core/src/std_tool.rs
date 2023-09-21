use crate::{Function, FunctionSet, Rush};
use anyhow::anyhow;
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;
use wd_tools::sync::Acl;
use wd_tools::{PFErr, PFOk};

#[derive(Default, Debug)]
pub struct Env {
    envs: Acl<HashMap<String, String>>,
}
impl Env {
    #[allow(dead_code)]
    pub fn add_env_variable<S: Into<String>>(&mut self, key: S, value: S) {
        self.envs.update(|arc| {
            let mut map = HashMap::new();
            for (k, v) in arc.iter() {
                map.insert(k.to_string(), v.to_string());
            }
            map.insert(key.into(), value.into());
            map
        });
    }
    #[allow(dead_code)]
    pub fn add_env_variables<Map: Into<HashMap<String, String>>>(&mut self, mp: Map) {
        self.envs.update(|arc| {
            let mut map = HashMap::new();
            for (k, v) in arc.iter() {
                map.insert(k.to_string(), v.to_string());
            }
            for (k, v) in mp.into() {
                map.insert(k, v);
            }
            map
        });
    }
}
impl From<HashMap<String, String>> for Env {
    fn from(envs: HashMap<String, String>) -> Self {
        let envs = Acl::new(envs);
        Env { envs }
    }
}
impl From<Acl<HashMap<String, String>>> for Env {
    fn from(envs: Acl<HashMap<String, String>>) -> Self {
        Env { envs }
    }
}
impl Rush {
    pub fn set_env<E: Into<Env>>(self, env: E) -> Self {
        self.raw_register_function("env", env.into())
    }
}

impl Function for Env {
    fn call(&self, _fs: Arc<dyn FunctionSet>, args: Vec<Value>) -> anyhow::Result<Value> {
        if args.len() < 1 {
            return anyhow!("must have a args").err();
        }
        if let Value::String(key) = args.get(0).unwrap() {
            if let Some(s) = self.envs.share().get(key) {
                return Value::String(s.to_string()).ok();
            }
        }
        return Value::String(String::new()).ok();
    }
}

pub struct ArrayContain;

impl ArrayContain {
    fn contain(array: &Vec<Value>, des: &Value) -> bool {
        match des {
            Value::Null => {
                for i in array {
                    if i.is_null() {
                        return true;
                    }
                }
            }
            Value::Bool(d) => {
                for i in array {
                    if let Some(s) = i.as_bool() {
                        if &s == d {
                            return true;
                        }
                    }
                }
            }
            Value::Number(n) => {
                if let Some(n) = n.as_i64() {
                    for i in array {
                        if let Some(i) = i.as_i64() {
                            if n == i {
                                return true;
                            }
                        }
                    }
                }
            }
            Value::String(s) => {
                for i in array {
                    if let Some(i) = i.as_str() {
                        if s == i {
                            return true;
                        }
                    }
                }
            }
            Value::Array(ref list) => {
                for i in list {
                    if !Self::contain(array, i) {
                        return false;
                    }
                }
                return true;
            }
            Value::Object(_) => {}
        };
        return false;
    }
}

impl Function for ArrayContain {
    fn call(&self, _fs: Arc<dyn FunctionSet>, mut args: Vec<Value>) -> anyhow::Result<Value> {
        if args.len() < 2 {
            return anyhow!("ArrayContain: must have two args").err();
        }
        let array = if let Value::Array(array) = args.remove(0) {
            array
        } else {
            return anyhow!("ArrayContain: first input must is array").err();
        };
        let des = args.remove(0);
        Ok(Value::Bool(ArrayContain::contain(&array, &des)))
    }
}

pub struct ArraySub;
impl Function for ArraySub {
    fn call(&self, _fs: Arc<dyn FunctionSet>, mut args: Vec<Value>) -> anyhow::Result<Value> {
        if args.len() < 2 {
            return anyhow!("ArrayContain: must have two args").err();
        }
        let la = if let Value::Array(array) = args.remove(0) {
            array
        } else {
            return anyhow!("ArrayContain: first input must is array").err();
        };
        let ra = if let Value::Array(array) = args.remove(0) {
            array
        } else {
            return anyhow!("ArrayContain: second input must is array").err();
        };
        for i in la {
            for j in &ra {
                if &i == j {
                    return Ok(Value::Bool(true));
                }
            }
        }
        return Ok(Value::Bool(false));
    }
}
