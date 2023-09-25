use crate::AsyncCustomScriptLoad;
use anyhow::anyhow;
use wd_tools::{PFErr, PFOk};

pub const LUA_SCRIPT_TAG: &'static str = "lua_script:";
pub const LUA_FILE_TAG: &'static str = "lua_file:";

pub struct AsyncCustomScriptLoadDefaultImpl;
#[async_trait::async_trait]
impl AsyncCustomScriptLoad for AsyncCustomScriptLoadDefaultImpl {
    async fn load(&self, _: String, mut script: String) -> anyhow::Result<String> {
        if !script.starts_with(LUA_SCRIPT_TAG) {
            return anyhow!(
                "AsyncCustomScriptLoadDefaultImpl: script start tag must is '{LUA_SCRIPT_TAG}'"
            )
            .err();
        }
        script.split_off(LUA_SCRIPT_TAG.len()).ok()
    }
}

pub struct AsyncCustomScriptLoadFile;

#[async_trait::async_trait]
impl AsyncCustomScriptLoad for AsyncCustomScriptLoadFile {
    async fn load(&self, _: String, mut script: String) -> anyhow::Result<String> {
        if !script.starts_with(LUA_FILE_TAG) {
            return anyhow!("AsyncCustomScriptLoadFile: script start tag must is '{LUA_FILE_TAG}'")
                .err();
        }
        let path = script.split_off(LUA_FILE_TAG.len());
        let path = path.trim_start_matches(|x| " \t\r\n".contains(x));
        let data = std::fs::read_to_string(path)?;
        data.ok()
    }
}
