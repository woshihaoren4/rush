#[cfg(test)]
mod test {
    use rush_core::AsyncRuleFlow;
    use rush_wasm_engine::WasmRuntimeFactory;
    use serde_json::Value;
    use std::collections::HashMap;

    const WASM_RULE: &'static str = "
    rule WASM_RULE _ wasm
    wasm_file: wasm_example/wasm_example_one.wasm
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

    const WASM_RULE_ERROR: &'static str = "
    rule WASM_RULE_ERROR _ wasm
    wasm_file: wasm_example/wasm_example_two.wasm
    ";
    #[tokio::test]
    async fn test_wasm_error_build() {
        let rt = WasmRuntimeFactory::new()
            .async_build(WASM_RULE_ERROR)
            .await
            .unwrap();

        let result: HashMap<String, String> =
            rt.async_flow(Value::String("true".into())).await.unwrap();
        assert_eq!(result.get("result").unwrap().as_str(), "success");

        let result: anyhow::Result<Value> = rt.async_flow(Value::String("false".into())).await;
        assert_eq!(
            result.unwrap_err().to_string().as_str(),
            "input[\"false\"] make a error"
        )
    }
}
