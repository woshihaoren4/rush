use crate::{
    AsyncCustomScriptLoadDefaultImpl, AsyncCustomScriptLoadFile, LuaRuntime, LUA_FILE_TAG,
    LUA_SCRIPT_TAG,
};
use anyhow::anyhow;
use std::collections::HashMap;
use wd_tools::PFErr;

#[async_trait::async_trait]
pub trait AsyncCustomScriptLoad: Send + Sync {
    //在同步中尝试解析
    fn try_load(&self, _rule_name: String, _script: String) -> Option<String> {
        None
    }
    async fn load(&self, rule_name: String, script: String) -> anyhow::Result<String>;
}

#[derive(Default)]
pub struct LuaRuntimeFactory {
    loader: HashMap<&'static str, Box<dyn AsyncCustomScriptLoad>>,
}

impl LuaRuntimeFactory {
    pub fn new() -> Self {
        let loader: HashMap<&'static str, Box<dyn AsyncCustomScriptLoad>> = HashMap::new();
        let mut lrf = Self { loader };
        lrf.add_loader(LUA_SCRIPT_TAG, AsyncCustomScriptLoadDefaultImpl);
        lrf.add_loader(LUA_FILE_TAG, AsyncCustomScriptLoadFile);
        lrf
    }
    pub fn add_loader<Load: AsyncCustomScriptLoad + 'static>(
        &mut self,
        tag: &'static str,
        loader: Load,
    ) {
        self.loader.insert(tag, Box::new(loader));
    }
    pub fn remove_loader<S: AsRef<str>>(&mut self, tag: S) {
        self.loader.remove(tag.as_ref());
    }
    fn check_engine(buf: &str) -> anyhow::Result<(String, String)> {
        let buf = buf.trim_start_matches(|c| " \n\r\t".contains(c));
        let (head, body) = if let Some(s) = buf.split_once('\n') {
            s
        } else {
            return anyhow!("first input must is : rule [name] [description] lua [other]").err();
        };
        let list = head.split(' ').collect::<Vec<_>>();
        if list.len() < 4 {
            return anyhow!("rule header format: rule [name] [description] lua [other]").err();
        }
        if list[0].to_lowercase() != "rule" {
            return anyhow!("rule header must have start 'rule'").err();
        }
        if list[3].to_lowercase() != "lua" {
            return anyhow!("LuaRuntime no support rule[{}]", list[3]).err();
        }
        let body = body.trim_start_matches(|c| " \n\r\t".contains(c));
        Ok((list[2].to_string(), body.into()))
    }
    pub fn load<S: AsRef<str>>(
        &self,
        script: S,
        envs: HashMap<String, String>,
    ) -> anyhow::Result<LuaRuntime> {
        let (rule, buf) = Self::check_engine(script.as_ref())?;
        for (k, v) in self.loader.iter() {
            if buf.starts_with(*k) {
                let script = if let Some(s) = v.try_load(rule, buf) {
                    s
                } else {
                    return anyhow!("can load script, please use async build").err();
                };
                return LuaRuntime::new(script, envs);
            }
        }
        anyhow!("not found eligible loader").err()
    }
    pub async fn build<S: AsRef<str>>(
        &self,
        script: S,
        envs: HashMap<String, String>,
    ) -> anyhow::Result<LuaRuntime> {
        let (rule, buf) = Self::check_engine(script.as_ref())?;
        for (k, v) in self.loader.iter() {
            if buf.starts_with(*k) {
                let script = v.load(rule, buf).await?;
                return LuaRuntime::new(script, envs);
            }
        }
        anyhow!("not found eligible loader").err()
    }
}

#[cfg(test)]
mod test {
    use crate::LuaRuntimeFactory;
    use serde_json::Value;
    use std::collections::HashMap;

    const LUA_RULE: &'static str = r#"
    rule LUA_RULE _ lua
    lua_script:
    function handle(req)
        for k, v in pairs(req) do
            print(prefix,"--->",k,v)
        end
        local resp = {message="success"}
        return resp
    end

    return {handle_function="handle"}
    "#;

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn test_factory() {
        let rt = LuaRuntimeFactory::new()
            .build(
                LUA_RULE,
                HashMap::from([("prefix".to_string(), "req".to_string())]),
            )
            .await
            .unwrap();
        let res: HashMap<String, String> = rt
            .async_call::<Value, _>(r#"{"like":"eat","age":18}"#.parse().unwrap())
            .await
            .unwrap();
        assert_eq!(res.get("message").unwrap().as_str(), "success")
    }
}
