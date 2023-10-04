use crate::ExportEnv;
use anyhow::{anyhow, Error};
use serde::{Deserialize, Serialize};
use std::time::Duration;
use tokio::sync::oneshot::{channel, error, Sender};
use wasmer::{imports, Function, FunctionEnv, Instance, Module, Store, TypedFunction, WasmPtr};
use wd_tools::PFErr;

#[derive(Debug)]
struct Task {
    input: Vec<u8>,
    sender: Sender<anyhow::Result<String>>,
}

pub struct WasmRuntime {
    sender: async_channel::Sender<Task>,
}

impl WasmRuntime {
    pub fn new(wasm_bytes: Vec<u8>) -> anyhow::Result<WasmRuntime> {
        let (sender, receiver) = async_channel::bounded(32);
        let (sender_init, mut receiver_init) = channel::<anyhow::Result<()>>();

        std::thread::spawn(move || {
            let mut store = Store::default();
            let module = match Module::new(&store, wasm_bytes) {
                Ok(m) => m,
                Err(e) => {
                    let _ = sender_init.send(Err(Error::from(e)));
                    return;
                }
            };

            let env = ExportEnv::new();
            let env = FunctionEnv::new(&mut store, env);
            let import_obj = imports! {
            "env"=>{
                "success" => Function::new_typed_with_env(&mut store,&env,ExportEnv::ret_success),
                "error" => Function::new_typed_with_env(&mut store,&env,ExportEnv::ret_error),
            }
            };

            let instance = match Instance::new(&mut store, &module, &import_obj) {
                Ok(o) => o,
                Err(e) => {
                    let _ = sender_init.send(Err(Error::from(e)));
                    return;
                }
            };

            let memory = match instance.exports.get_memory("memory") {
                Ok(o) => o,
                Err(e) => {
                    let _ = sender_init.send(Err(Error::from(e)));
                    return;
                }
            };
            env.as_mut(&mut store).init(memory.clone());

            let handle: TypedFunction<(WasmPtr<u8>, u32), u32> =
                match instance.exports.get_typed_function(&store, "handle") {
                    Ok(o) => o,
                    Err(e) => {
                        let _ = sender_init.send(Err(Error::from(e)));
                        return;
                    }
                };

            if sender_init.send(Ok(())).is_err() {
                return;
            }

            loop {
                let Task { input, sender } = match receiver.recv_blocking() {
                    Ok(o) => o,
                    Err(_) => return,
                };
                let view = memory.view(&store);
                let ptr: WasmPtr<u8> = WasmPtr::new(0);
                let value = match ptr.slice(&view, input.len() as u32) {
                    Ok(o) => o,
                    Err(e) => {
                        let _ = sender.send(Err(Error::from(e)));
                        continue;
                    }
                };
                if let Err(e) = value.write_slice(input.as_slice()) {
                    let _ = sender.send(Err(Error::from(e)));
                    continue;
                };
                if let Err(e) = handle.call(&mut store, ptr, input.len() as u32) {
                    let _ = sender.send(Err(Error::from(e)));
                    continue;
                }
                let result = env.as_mut(&mut store).get_result();
                let _ = sender.send(result);
            }
        });

        loop {
            match receiver_init.try_recv() {
                Ok(Ok(_)) => {
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
        Ok(Self { sender })
    }

    pub fn call<S: Serialize, Out: for<'a> Deserialize<'a>>(&self, req: S) -> anyhow::Result<Out> {
        let input = serde_json::to_string(&req)?.into_bytes();
        let (sender, receiver) = channel();
        let task = Task { input, sender };
        if let Err(e) = self.sender.send_blocking(task) {
            let err = e.to_string();
            return anyhow!("wasm runtime call failed: {}", err).err();
        }
        return match receiver.blocking_recv() {
            Ok(o) => {
                let s = o?;
                let out = serde_json::from_str::<Out>(s.as_str())?;
                Ok(out)
            }
            Err(e) => anyhow!("wasm runtime error:{}", e).err(),
        };
    }

    pub async fn async_call<S: Serialize, Out: for<'a> Deserialize<'a>>(
        &self,
        req: S,
    ) -> anyhow::Result<Out> {
        let input = serde_json::to_string(&req)?.into_bytes();
        let (sender, receiver) = channel();
        let task = Task { input, sender };
        if let Err(e) = self.sender.send(task).await {
            let err = e.to_string();
            return anyhow!("lua runtime call failed: {}", err).err();
        }
        return match receiver.await {
            Ok(o) => {
                let s = o?;
                let out = serde_json::from_str::<Out>(s.as_str())?;
                Ok(out)
            }
            Err(e) => anyhow!("wasm runtime error:{}", e).err(),
        };
    }
}
impl Drop for WasmRuntime {
    fn drop(&mut self) {
        self.sender.close();
    }
}

impl<T: Into<Vec<u8>>> From<T> for WasmRuntime {
    fn from(value: T) -> Self {
        WasmRuntime::new(value.into()).unwrap()
    }
}

#[cfg(feature = "rule-flow")]
impl rush_core::RuleFlow for WasmRuntime {
    fn flow<Obj: Serialize, Out: for<'a> Deserialize<'a>>(&self, obj: Obj) -> anyhow::Result<Out> {
        self.call(obj)
    }
}
#[cfg(feature = "rule-flow")]
#[async_trait::async_trait]
impl rush_core::AsyncRuleFlow for WasmRuntime {
    async fn async_flow<Obj: Serialize + Send, Out: for<'a> Deserialize<'a>>(
        &self,
        obj: Obj,
    ) -> anyhow::Result<Out> {
        self.async_call(obj).await
    }
}
#[cfg(test)]
mod test {
    use crate::WasmRuntime;
    use serde_json::Value;
    use std::collections::HashMap;

    #[test]
    pub fn test_wasm() {
        let wasm_bytes =
            include_bytes!("../../target/wasm32-unknown-unknown/release/wasm_example_one.wasm");

        let wr = WasmRuntime::new(wasm_bytes.to_vec()).unwrap();

        let result: HashMap<String, String> = wr.call(Value::String("hello".into())).unwrap();
        assert_eq!(result.get("input").unwrap().as_str(), "hello");

        let result: HashMap<String, String> = wr.call(Value::String("world".into())).unwrap();
        assert_eq!(result.get("input").unwrap().as_str(), "world");
    }
}
