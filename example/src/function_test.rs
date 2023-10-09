#[cfg(test)]
mod test {
    use rush_core::{Function, FunctionSet, RuleFlow, Rush};
    use rush_expr_engine::ExprEngine;
    use serde::{Deserialize, Serialize};
    use serde_json::Value;
    use std::sync::Arc;

    const FUNCTION_RULE: &'static str = "
    rule FUNCTION_RULE
    when
        str_rev(country)=='加拿大';
        str_splice(str_rev(country),city)=='加拿大 多伦多';
        abs(revenue.low) < 100 && abs(revenue.high) > 1000;
    then
        message = str_splice(str_rev(country),city,'贫富差距大');
        revenue.avg = abs(revenue.low) + abs(revenue.high) / 2;
    ";

    //可以是函数
    pub fn str_rev(s: String) -> anyhow::Result<String> {
        let mut result = String::new();
        let si = s.chars();
        for s in si.rev() {
            result.push(s);
        }
        Ok(result)
    }

    //也可以实现了trait的struct
    pub struct StrSplice(char);
    impl Function for StrSplice {
        fn call(&self, _fs: Arc<dyn FunctionSet>, args: Vec<Value>) -> anyhow::Result<Value> {
            let mut res = String::new();
            for i in args {
                if !res.is_empty() {
                    res.push(self.0);
                }
                if let Value::String(s) = i {
                    res += s.as_str();
                }
            }
            Ok(Value::String(res))
        }
    }

    #[derive(Deserialize, Serialize)]
    pub struct Resp {
        #[serde(default = "Default::default")]
        message: String,
        #[serde(default = "Default::default")]
        revenue: Revenue,
    }
    #[derive(Deserialize, Serialize, Default)]
    pub struct Revenue {
        avg: i64,
    }

    #[test]
    fn test_function() {
        let ee = ExprEngine::from([FUNCTION_RULE]);
        let rh = Rush::from(ee);

        let rh = rh
            .raw_register_function("str_splice", StrSplice(' '))
            .register_function("str_rev", str_rev)
            //可以用闭包的方式添加
            .register_function("abs", |i: i64| Ok(i.abs()));

        let resp: Resp = rh
            .flow(
                r#"{"country":"大拿加","city":"多伦多","revenue":{"low":-10,"high":1002}}"#
                    .parse::<Value>()
                    .unwrap(),
            )
            .unwrap();
        assert_eq!(resp.message.as_str(), "加拿大 多伦多 贫富差距大");
        assert_eq!(resp.revenue.avg, 506);

        let resp: Resp = rh
            .flow(
                r#"{"country":"加拿大","city":"多伦多","revenue":{"low":-10,"high":1002}}"#
                    .parse::<Value>()
                    .unwrap(),
            )
            .unwrap();
        assert_eq!(resp.message.as_str(), "");
        assert_eq!(resp.revenue.avg, 0);

        let resp: Resp = rh
            .flow(
                r#"{"country":"大拿加","city":"温哥华","revenue":{"low":-10,"high":1002}}"#
                    .parse::<Value>()
                    .unwrap(),
            )
            .unwrap();
        assert_eq!(resp.message.as_str(), "");
        assert_eq!(resp.revenue.avg, 0);

        let resp: Resp = rh
            .flow(
                r#"{"country":"加拿大","city":"多伦多","revenue":{"low":-100,"high":1002}}"#
                    .parse::<Value>()
                    .unwrap(),
            )
            .unwrap();
        assert_eq!(resp.message.as_str(), "");
        assert_eq!(resp.revenue.avg, 0);
    }
}
