#[cfg(test)]
mod test{
    use serde_json::{Map, Value};
    use expr_engine::ExprEngine;
    use rush_core::{Filter, Rush};
    use serde::Deserialize;

    pub const NULL_EXPR_RULE:&str = "
    rule NULL_EXPR_RULE
    when
    then
    ";
    #[test]
    fn test_null_success(){
        let ee = ExprEngine::from([NULL_EXPR_RULE]);
        let rh = Rush::from(ee);
        let val:Value = rh.input(()).unwrap();
        assert_eq!(val,Value::Object(Map::new()),"if when is null,then all always success");
    }

    pub const NULL_WHEN_ONE_EXEC:&str = "
    rule NULL_WHEN_ONE_EXEC
    when
    then
        message = 'success'
    ";
    #[test]
    #[should_panic]
    fn test_null_failed(){
        let ee = ExprEngine::from([NULL_WHEN_ONE_EXEC]);
        let rh = Rush::from(ee);
        let val:Value = rh.input(()).unwrap();
        assert_eq!(val,Value::Object(Map::new()),"if when is null,then all always success");
    }

    pub const MANY_RULE_ONE:&str = "
    rule MANY_RULE_ONE
    when
        country == '美国';
        age <= 18;
    then
        tag = '美国的年轻人'
    ";

    pub const MANY_RULE_TWO:&str = "
    rule MANY_RULE_TWO
    when
        country == '美国';
        age > 18 && age < 30;
    then
        tag = '美国的青年人'
    ";

    pub const MANY_RULE_THREE:&str = "
    rule MANY_RULE_THREE
    when
        country == '中国';
        age <= 18;
    then
        tag = '中国的年轻人'
    ";
    pub const MANY_RULE_FOUR:&str = "
    rule MANY_RULE_FOUR
    when
        country == '中国';
        age > 18 && age < 30;
    then
        tag = '中国的青年人'
    ";

    #[derive(Deserialize)]
    struct Tag{
        #[serde(default="Default::default")]
        tag:String
    }
    #[test]
    fn test_many_test(){
        let rh = Rush::from(Into::<ExprEngine>::into([MANY_RULE_ONE,MANY_RULE_TWO,MANY_RULE_THREE,MANY_RULE_FOUR]));
        let res:Tag = rh.input(r#"{"country":"美国","age":17}"#.parse::<Value>().unwrap()).unwrap();
        assert_eq!(res.tag.as_str(),"美国的年轻人",r#"case : {{"country":"美国","age":17}} failed"#);
        let res:Tag = rh.input(r#"{"country":"美国","age":19}"#.parse::<Value>().unwrap()).unwrap();
        assert_eq!(res.tag.as_str(),"美国的青年人",r#"case : {{"country":"美国","age":19}} failed"#);
        let res:Tag = rh.input(r#"{"country":"中国","age":17}"#.parse::<Value>().unwrap()).unwrap();
        assert_eq!(res.tag.as_str(),"中国的年轻人",r#"case : {{"country":"中国","age":17}} failed"#);
        let res:Tag = rh.input(r#"{"country":"中国","age":19}"#.parse::<Value>().unwrap()).unwrap();
        assert_eq!(res.tag.as_str(),"中国的青年人",r#"case : {{"country":"中国","age":19}} failed"#);

        let res:Tag = rh.input(r#"{"age":17}"#.parse::<Value>().unwrap()).unwrap();
        assert_eq!(res.tag.as_str(),"",r#"case: country is null failed"#);
        let res:Tag = rh.input(r#"{"country":"美国"}"#.parse::<Value>().unwrap()).unwrap();
        assert_eq!(res.tag.as_str(),"",r#"case: age is null failed"#);

    }
}