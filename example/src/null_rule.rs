#[cfg(test)]
mod test{
    use serde_json::{Map, Value};
    use expr_engine::ExprEngine;
    use rush_core::{Filter, Rush};

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
}