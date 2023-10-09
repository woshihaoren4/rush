use anyhow::{anyhow, Error};
use std::mem::MaybeUninit;
use wasmer::{imports, Function, FunctionEnv, FunctionEnvMut, Imports, Memory, Store, WasmPtr};
use wd_tools::PFErr;

pub struct ExportEnv {
    memory: MaybeUninit<Memory>,
    result: anyhow::Result<String>,
}

impl ExportEnv {
    pub fn new() -> Self {
        Self {
            memory: MaybeUninit::uninit(),
            result: anyhow!("not init").err(),
        }
    }
    pub fn init(&mut self, mem: Memory) {
        self.memory.write(mem);
    }

    //成功并且返回
    pub fn ret_success(mut env: FunctionEnvMut<ExportEnv>, ptr: WasmPtr<u8>, len: u32) -> u32 {
        let (env, store) = env.data_and_store_mut();
        unsafe {
            let view = env.memory.assume_init_ref().view(&store);
            let result = ptr.read_utf8_string(&view, len);
            env.result = match result {
                Ok(o) => Ok(o),
                Err(e) => Err(Error::from(e)),
            };
        }
        0u32
    }
    //失败返回
    pub fn ret_error(mut env: FunctionEnvMut<ExportEnv>, ptr: WasmPtr<u8>, len: u32) -> u32 {
        let (env, store) = env.data_and_store_mut();
        unsafe {
            let view = env.memory.assume_init_ref().view(&store);
            let result = ptr.read_utf8_string(&view, len);
            env.result = match result {
                Ok(o) => anyhow!("{}", o).err(),
                Err(e) => Err(Error::from(e)),
            };
        }
        0u32
    }
    pub fn generate_import(env: &FunctionEnv<Self>, store: &mut Store) -> Imports {
        imports! {
             "env"=>{
                 "success" => Function::new_typed_with_env(store,env,ExportEnv::ret_success),
                 "error" => Function::new_typed_with_env(store,env,ExportEnv::ret_error),
             }
        }
    }
    pub fn get_result(&mut self) -> anyhow::Result<String> {
        unsafe { std::ptr::replace(&mut self.result, anyhow!("not init").err()) }
    }
}
