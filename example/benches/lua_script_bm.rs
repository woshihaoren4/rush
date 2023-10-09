use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};
use rush_core::AsyncRuleFlow;
use rush_lua_engine::{LuaRuntime, LuaRuntimeFactory};
use serde::Deserialize;
use serde_json::Value;
use std::collections::HashMap;

const LUA_RULE_SCRIPT: &'static str = r#"
    rule LUA_RULE_SCRIPT _ lua
    lua_script:
    function handle(req)
        local resp = {}

        if req.source == ONLINE_CHANNEL then
            resp.message = "线上渠道"
        elseif req.source == OFFLINE_CHANNEL then
            resp.message = "线下渠道"
        else
            resp.message = "未知渠道:"..req.source
        end

        return resp
    end

    return {handle_function="handle"}
    "#;
#[derive(Deserialize)]
struct Resp {
    #[serde(default = "Default::default")]
    message: String,
}

async fn lua_async_flow(rt: &LuaRuntime) {
    let res: Resp = rt
        .async_flow(r#"{"source":"online"}"#.parse::<Value>().unwrap())
        .await
        .unwrap();
    assert_eq!(res.message.as_str(), "线上渠道");
}

fn async_lua_benchmark(c: &mut Criterion) {
    let mut envs = HashMap::new();
    envs.insert("ONLINE_CHANNEL".into(), "online".into());
    envs.insert("OFFLINE_CHANNEL".into(), "offline".into());

    let rt = LuaRuntimeFactory::new()
        .load(LUA_RULE_SCRIPT, envs)
        .unwrap();

    c.bench_with_input(
        BenchmarkId::new("lua_async_flow", "lua_async_flow"),
        &rt,
        |b, s| {
            b.to_async(tokio::runtime::Runtime::new().unwrap())
                .iter(|| lua_async_flow(s));
        },
    );
}

criterion_group!(benches, async_lua_benchmark);
criterion_main!(benches);
