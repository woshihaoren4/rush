#[cfg(test)]
mod test {
    use rush_core::AsyncRuleFlow;
    use serde_json::Value;
    use std::collections::HashMap;
    use wasm_engine::WasmRuntimeFactory;

    const WASM_RULE: &'static str = "
    rule WASM_RULE _ wasm
    wasm_file: ../target/wasm32-unknown-unknown/release/wasm_example_one.wasm
    ";

    #[tokio::test]
    async fn test_wasm_build() {
        let rt = WasmRuntimeFactory::new()
            .async_build(WASM_RULE)
            .await
            .unwrap();

        let result: HashMap<String, String> =
            rt.async_flow(Value::String("hello".into())).await.unwrap();
        assert_eq!(result.get("input").unwrap().as_str(), "hello");
    }
}
