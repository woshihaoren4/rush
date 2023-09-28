use anyhow::{anyhow, Error};
use mlua::{Function, Lua, LuaSerdeExt, Value};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::str::FromStr;
use std::time::Duration;
use tokio::sync::oneshot::*;
use wd_tools::{PFErr, PFOk};

#[derive(Debug)]
struct Task {
    input: serde_json::Value,
    sender: Sender<anyhow::Result<serde_json::Value>>,
}

#[derive(Serialize, Deserialize, Default)]
pub struct InitResult {
    #[serde(default = "Default::default")]
    code: isize,
    #[serde(default = "String::default")]
    message: String,
    #[serde(default = "String::default")]
    handle_function: String,
}

#[derive(Debug)]
pub struct LuaRuntime {
    sender: async_channel::Sender<Task>,
}

impl Clone for LuaRuntime {
    fn clone(&self) -> Self {
        Self {
            sender: self.sender.clone(),
        }
    }
}

impl LuaRuntime {
    pub fn new(script: String, envs: HashMap<String, String>) -> anyhow::Result<LuaRuntime> {
        let (sender, receiver) = async_channel::bounded(32);
        let (sender_init, mut receiver_init) = channel::<anyhow::Result<InitResult>>();

        std::thread::spawn(move || {
            let lua = Lua::new();
            let global = lua.globals();

            for (k, v) in envs {
                if let Err(e) = global.set(k, v) {
                    let _ = sender_init.send(Err(Error::from(e)));
                    return;
                }
            }

            let result = lua.load(script).eval::<Option<Value>>();
            let val = match result {
                Ok(o) => {
                    if o.is_none() {
                        let _ = sender_init.send(
                            anyhow!("load script init failed, not return check result").err(),
                        );
                        return;
                    }
                    o.unwrap()
                }
                Err(e) => {
                    let _ = sender_init.send(Err(Error::from(e)));
                    return;
                }
            };
            let result = match lua.from_value::<InitResult>(val) {
                Ok(o) => o,
                Err(e) => {
                    let _ = sender_init.send(Err(Error::from(e)));
                    return;
                }
            };
            if result.code != 0 {
                let _ = sender_init.send(Ok(result));
                return;
            }
            if result.handle_function.is_empty() {
                let _ = sender_init.send(anyhow!("entry function must manifest").err());
                return;
            }
            let func = match global.get::<_, Function>(result.handle_function.as_str()) {
                Ok(f) => f,
                Err(e) => {
                    let _ = sender_init.send(Err(Error::from(e)));
                    return;
                }
            };

            if sender_init.send(InitResult::default().ok()).is_err() {
                return;
            }
            loop {
                let Task { input, sender } = match receiver.recv_blocking() {
                    Ok(o) => o,
                    Err(_) => return,
                };
                let val = match lua.to_value(&input) {
                    Ok(o) => o,
                    Err(e) => {
                        let _ = sender.send(Err(Error::from(e)));
                        continue;
                    }
                };
                let val = match func.call::<_, Value>(val) {
                    Ok(o) => o,
                    Err(e) => {
                        let _ = sender.send(Err(Error::from(e)));
                        continue;
                    }
                };

                match lua.from_value::<serde_json::Value>(val) {
                    Ok(o) => {
                        let _ = sender.send(Ok(o));
                    }
                    Err(e) => {
                        let _ = sender.send(Err(Error::from(e)));
                    }
                };
            }
        });

        loop {
            match receiver_init.try_recv() {
                Ok(Ok(o)) => {
                    if o.code != 0 {
                        return anyhow!("init lua runtime failed:{}", o.message).err();
                    }
                    break;
                }
                Ok(Err(e)) => {
                    return Err(e);
                }
                Err(error::TryRecvError::Closed) => {
                    return anyhow!("init lua runtime unknown error").err();
                }
                Err(error::TryRecvError::Empty) => {
                    std::thread::sleep(Duration::from_millis(1));
                }
            }
        }
        Ok(LuaRuntime { sender })
    }

    pub fn call<S: Serialize, Out: for<'a> Deserialize<'a>>(&self, req: S) -> anyhow::Result<Out> {
        let req = serde_json::to_value(req)?;
        let (sender, receiver) = channel();
        let task = Task { input: req, sender };
        if let Err(e) = self.sender.send_blocking(task) {
            let err = e.to_string();
            return anyhow!("lua runtime call failed: {}", err).err();
        }
        return match receiver.blocking_recv() {
            Ok(o) => {
                let out = serde_json::from_value::<Out>(o?)?;
                Ok(out)
            }
            Err(e) => anyhow!("lua runtime error:{}", e).err(),
        };
    }
    pub async fn async_call<S: Serialize, Out: for<'a> Deserialize<'a>>(
        &self,
        req: S,
    ) -> anyhow::Result<Out> {
        let req = serde_json::to_value(req)?;
        let (sender, receiver) = channel();
        let task = Task { input: req, sender };
        if let Err(e) = self.sender.send(task).await {
            let err = e.to_string();
            return anyhow!("lua runtime call failed: {}", err).err();
        }
        return match receiver.await {
            Ok(o) => {
                let out: Out = serde_json::from_value(o?)?;
                Ok(out)
            }
            Err(e) => anyhow!("lua runtime error:{}", e).err(),
        };
    }
    pub fn close(&self) {
        self.sender.close();
    }
}

impl Drop for LuaRuntime {
    fn drop(&mut self) {
        self.close();
    }
}

#[cfg(feature = "rule-flow")]
impl rush_core::RuleFlow for LuaRuntime {
    fn flow<Obj: Serialize, Out: for<'a> Deserialize<'a>>(&self, obj: Obj) -> anyhow::Result<Out> {
        self.call(obj)
    }
}
#[cfg(feature = "rule-flow")]
#[async_trait::async_trait]
impl rush_core::AsyncRuleFlow for LuaRuntime {
    async fn async_flow<Obj: Serialize + Send, Out: for<'a> Deserialize<'a>>(
        &self,
        obj: Obj,
    ) -> anyhow::Result<Out> {
        self.async_call(obj).await
    }
}

impl FromStr for LuaRuntime {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        LuaRuntime::new(s.to_string(), HashMap::new())
    }
}

#[cfg(test)]
mod test {
    use crate::LuaRuntime;
    use serde_json::Value;
    use std::collections::HashMap;

    const TEST_LUA_SCRIPT: &'static str = r#"
    function handle(req)
        for k, v in pairs(req) do
            print("--->",k,v)
        end
        local resp = "success"
        return resp
    end

    return {code=0,message="success",handle_function="handle"}
    "#;

    #[test]
    fn test_function_lua_runtime() {
        let rt = LuaRuntime::new(TEST_LUA_SCRIPT.to_string(), HashMap::new()).unwrap();
        let result: Value = rt
            .call(r#"{"order_id":"78632839429034208","status":1}"#.parse::<Value>().unwrap())
            .unwrap();
        assert_eq!(result, Value::String("success".into()))
    }
}
