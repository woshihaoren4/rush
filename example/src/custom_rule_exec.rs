#[cfg(test)]
mod test{
    use std::collections::HashMap;
    use std::sync::Arc;
    use serde_json::Value;
    use rush_core::{CalcNode, Exec, Filter, FunctionSet, Rush};

    struct CustomCalc;
    impl CalcNode for CustomCalc{
        fn when(&self, _fs: Arc<dyn FunctionSet>, input: &Value) -> anyhow::Result<bool> {
            if let Value::String(s) = input{
                return Ok(s == "true")
            }
            return Ok(false)
        }
    }
    struct CustomExec;
    impl Exec for CustomExec{
        fn execute(&self, _fs: Arc<dyn FunctionSet>, _input: &Value, output: &mut Value) -> anyhow::Result<()> {
            if let Value::Object(obj) = output{
                obj.insert("result".to_string(),Value::from("success"));
            }
            Ok(())
        }
    }

    #[test]
    fn test_custom_calc_exec(){
        let rh = Rush::new()
            .register_rule("custom_rule",vec![CustomCalc],CustomExec);
        let res:HashMap<String,String> = rh.flow("true".parse::<String>().unwrap()).unwrap();
        assert_eq!(res.get("result").unwrap().as_str(),"success");

        let res:HashMap<String,String> = rh.flow("false".parse::<String>().unwrap()).unwrap();
        assert_eq!(res.get("result"),None);
    }
}