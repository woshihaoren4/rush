use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};
use rush_core::AsyncRuleFlow;
use serde::Deserialize;
use serde_json::Value;
use std::collections::HashMap;
use wasm_engine::{WasmRuntime, WasmRuntimeFactory};

const WASM_RULE: &'static str = "
    rule WASM_RULE _ wasm
    wasm_file: ../target/wasm32-unknown-unknown/release/wasm_example_one.wasm
    ";

#[derive(Deserialize)]
struct Resp {
    #[serde(default = "Default::default")]
    input: String,
}

async fn wasm_async_flow(rt: &WasmRuntime) {
    let res: Resp = rt
        .async_flow(Value::String("hello world".into()))
        .await
        .unwrap();
    assert_eq!(res.input.as_str(), "hello world");
}

fn async_wasm_benchmark(c: &mut Criterion) {
    let rt = WasmRuntimeFactory::new().build(WASM_RULE).unwrap();

    c.bench_with_input(
        BenchmarkId::new("wasm_async_flow", "wasm_async_flow"),
        &rt,
        |b, s| {
            b.to_async(tokio::runtime::Runtime::new().unwrap())
                .iter(|| wasm_async_flow(s));
        },
    );
}

criterion_group!(benches, async_wasm_benchmark);
criterion_main!(benches);
