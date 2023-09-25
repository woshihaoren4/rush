#[cfg(test)]
mod test {
    use lua_engine::{LuaRuntime, LuaRuntimeFactory};
    use rush_core::{AsyncRuleFlow, RuleFlow};
    use serde_json::Value;
    use std::collections::HashMap;

    const LUA_SCRIPT: &'static str = r#"
    function handle(req)
        local resp = {}

        if req.source == "online" then
            resp.message = "线上渠道"
        elseif req.source == "offline" then
            resp.message = "线下渠道"
        else
            resp.message = "未知渠道:"..req.source
        end

        return resp
    end

    return {handle_function="handle"}
    "#;

    #[test]
    fn test_lua_time() {
        let rt = LUA_SCRIPT.parse::<LuaRuntime>().unwrap();

        let res: HashMap<String, String> = rt
            .flow(r#"{"source":"online"}"#.parse::<Value>().unwrap())
            .unwrap();
        assert_eq!(res.get("message").unwrap().as_str(), "线上渠道");

        let res: HashMap<String, String> = rt
            .flow(r#"{"source":"offline"}"#.parse::<Value>().unwrap())
            .unwrap();
        assert_eq!(res.get("message").unwrap().as_str(), "线下渠道");

        let res: HashMap<String, String> = rt
            .flow(r#"{"source":"unknown"}"#.parse::<Value>().unwrap())
            .unwrap();
        assert_eq!(res.get("message").unwrap().as_str(), "未知渠道:unknown");
    }

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

    #[tokio::test]
    async fn test_lua_rule_build() {
        let mut envs = HashMap::new();
        envs.insert("ONLINE_CHANNEL".into(), "online".into());
        envs.insert("OFFLINE_CHANNEL".into(), "offline".into());

        let rt = LuaRuntimeFactory::new()
            .build(LUA_RULE_SCRIPT, envs)
            .await
            .unwrap();

        let res: HashMap<String, String> = rt
            .async_flow(r#"{"source":"online"}"#.parse::<Value>().unwrap())
            .await
            .unwrap();
        assert_eq!(res.get("message").unwrap().as_str(), "线上渠道");
    }
}
