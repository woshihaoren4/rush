#[cfg(test)]
mod test {
    use expr_engine::ExprEngine;
    use rush_core::{Filter, Rush};
    use serde_json::Value;
    use std::collections::HashMap;

    const ARRAY_RULE:&'static str = "
    rule ARRAY_RULE
    when
        contain([1,2,3,4],status) && !contain([5],status);
        contain([2,3.1,'hello',true,2>>1],1) && !contain([2,3.1,'hello',true,2>>1],3.1) /*浮点数无法判断包含*/;
        sub([1,2,3,4],[1]) && !sub([1,2,3,4],[5]);
        sub([2,3.1,'hello',true,2>>1],[1,'world']) && !contain([2,3.1,'hello',true,2>>1],[3.1]) /*浮点数无法判断子集*/;
    then
        message = 'success'
    ";

    #[test]
    fn test_array_rule() {
        let rh = Rush::from(Into::<ExprEngine>::into([ARRAY_RULE]));

        let res: HashMap<String, String> = rh
            .flow(r#"{"status":2}"#.parse::<Value>().unwrap())
            .unwrap();
        assert_eq!(res.get("message").unwrap().as_str(), "success");
    }
    const ENV_RULE_DISCOUNT: &'static str = "
    rule ENV_RULE_DISCOUNT
    when
        env('ACTIVITY_DISCOUNT_TYPE') == type
    then
        type = '打折活动'
    ";
    const ENV_RULE_COUPON: &'static str = "
    rule ENV_RULE_COUPON
    when
        env('ACTIVITY_COUPON_TYPE') == type
    then
        type = '卡券活动'
    ";
    #[test]
    fn test_env_rule() {
        let envs = HashMap::from([
            ("ACTIVITY_DISCOUNT_TYPE".to_string(), "discount".to_string()),
            ("ACTIVITY_COUPON_TYPE".to_string(), "coupon".to_string()),
        ]);

        let rh = Rush::from(Into::<ExprEngine>::into([
            ENV_RULE_DISCOUNT,
            ENV_RULE_COUPON,
        ]))
        .set_env(envs);

        let res: HashMap<String, String> = rh
            .flow(r#"{"type":"discount"}"#.parse::<Value>().unwrap())
            .unwrap();
        assert_eq!(res.get("type").unwrap().as_str(), "打折活动");
        let res: HashMap<String, String> = rh
            .flow(r#"{"type":"coupon"}"#.parse::<Value>().unwrap())
            .unwrap();
        assert_eq!(res.get("type").unwrap().as_str(), "卡券活动");
    }
}
