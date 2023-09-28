use crate::AsyncCustomScriptLoad;
use anyhow::anyhow;
use wd_tools::{PFErr, PFOk, PFSome};

pub const LUA_SCRIPT_TAG: &'static str = "lua_script:";
pub const LUA_FILE_TAG: &'static str = "lua_file:";

pub struct AsyncCustomScriptLoadDefaultImpl;
#[async_trait::async_trait]
impl AsyncCustomScriptLoad for AsyncCustomScriptLoadDefaultImpl {
    fn try_load(&self, _rule_name: String, mut script: String) -> Option<String> {
        if !script.starts_with(LUA_SCRIPT_TAG) {
            return None;
        }
        script.split_off(LUA_SCRIPT_TAG.len()).some()
    }

    async fn load(&self, rule_name: String, script: String) -> anyhow::Result<String> {
        return match self.try_load(rule_name, script) {
            None => anyhow!(
                "AsyncCustomScriptLoadDefaultImpl: script start tag must is '{LUA_SCRIPT_TAG}'"
            )
            .err(),
            Some(s) => Ok(s),
        };
    }
}

pub struct AsyncCustomScriptLoadFile;

#[async_trait::async_trait]
impl AsyncCustomScriptLoad for AsyncCustomScriptLoadFile {
    fn try_load(&self, _rule_name: String, mut script: String) -> Option<String> {
        if !script.starts_with(LUA_FILE_TAG) {
            return None;
        }
        let path = script.split_off(LUA_FILE_TAG.len());
        let path = path.trim_matches(|x| " \t\r\n".contains(x));
        let data = match std::fs::read_to_string(path) {
            Ok(s) => s,
            Err(e) => {
                println!(
                    "AsyncCustomScriptLoadFile open file failed; {}",
                    e.to_string()
                );
                return None;
            }
        };
        data.some()
    }

    async fn load(&self, _rule_name: String, mut script: String) -> anyhow::Result<String> {
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
