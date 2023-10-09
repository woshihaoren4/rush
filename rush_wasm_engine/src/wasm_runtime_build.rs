use crate::{WasmLoaderFile, WasmRuntime, WASM_LOADER_FILE};
use anyhow::anyhow;
use std::collections::HashMap;
use wd_tools::PFErr;

#[async_trait::async_trait]
pub trait WasmLoader: Send + Sync {
    fn load(&self, _rule_name: String, file: String) -> anyhow::Result<Vec<u8>>;
    async fn async_load(&self, rule_name: String, file: String) -> anyhow::Result<Vec<u8>> {
        self.load(rule_name, file)
    }
}

#[derive(Default)]
pub struct WasmRuntimeFactory {
    loader: HashMap<&'static str, Box<dyn WasmLoader>>,
}

impl WasmRuntimeFactory {
    pub fn new() -> Self {
        let loader: HashMap<&'static str, Box<dyn WasmLoader>> = HashMap::new();
        let mut lrf = Self { loader };
        lrf.add_loader(WASM_LOADER_FILE, WasmLoaderFile);
        lrf
    }
    pub fn add_loader<Load: WasmLoader + 'static>(&mut self, tag: &'static str, loader: Load) {
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
            return anyhow!("first input must is : rule [name] [description] wasm [other]").err();
        };
        let list = head.split(' ').collect::<Vec<_>>();
        if list.len() < 4 {
            return anyhow!("rule header format: rule [name] [description] wasm [other]").err();
        }
        if list[0].to_lowercase() != "rule" {
            return anyhow!("rule header must have start 'rule'").err();
        }
        if list[3].to_lowercase() != "wasm" {
            return anyhow!("WasmRuntime no support rule[{}]", list[3]).err();
        }
        let body = body.trim_start_matches(|c| " \n\r\t".contains(c));
        Ok((list[2].to_string(), body.into()))
    }
    pub fn build<S: AsRef<str>>(&self, rule: S) -> anyhow::Result<WasmRuntime> {
        let (rule, buf) = Self::check_engine(rule.as_ref())?;
        for (k, v) in self.loader.iter() {
            if buf.starts_with(*k) {
                let bytes = v.load(rule, buf)?;
                return WasmRuntime::new(bytes);
            }
        }
        anyhow!("not found eligible loader").err()
    }
    pub async fn async_build<S: AsRef<str>>(&self, rule: S) -> anyhow::Result<WasmRuntime> {
        let (rule, buf) = Self::check_engine(rule.as_ref())?;
        for (k, v) in self.loader.iter() {
            if buf.starts_with(*k) {
                let bytes = v.async_load(rule, buf).await?;
                return WasmRuntime::new(bytes);
            }
        }
        anyhow!("not found eligible loader").err()
    }
}

#[cfg(test)]
mod test {
    use crate::WasmRuntimeFactory;
    use serde_json::Value;
    use std::collections::HashMap;

    const WASM_RULE: &'static str = "
    rule WASM_RULE _ wasm
    wasm_file: ../target/wasm32-unknown-unknown/release/wasm_example_one.wasm
    ";

    #[test]
    fn test_wasm_build() {
        let rt = WasmRuntimeFactory::new().build(WASM_RULE).unwrap();

        let result: HashMap<String, String> = rt.call(Value::String("hello".into())).unwrap();
        assert_eq!(result.get("input").unwrap().as_str(), "hello");
    }
}
