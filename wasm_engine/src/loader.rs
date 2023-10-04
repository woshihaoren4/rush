use crate::WasmLoader;
use anyhow::anyhow;
use wd_tools::{PFErr, PFOk};

pub const WASM_LOADER_FILE: &'static str = "wasm_file:";
pub struct WasmLoaderFile;

impl WasmLoader for WasmLoaderFile {
    fn load(&self, _rule_name: String, mut file: String) -> anyhow::Result<Vec<u8>> {
        if !file.starts_with(WASM_LOADER_FILE) {
            return anyhow!("expect loader tag:{} not found", WASM_LOADER_FILE).err();
        }
        let path = file.split_off(WASM_LOADER_FILE.len());
        let path = path.trim_matches(|x| " \t\r\n".contains(x));
        let bytes = std::fs::read(path)?;
        bytes.ok()
    }
}
