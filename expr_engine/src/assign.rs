use std::collections::HashMap;
use std::str::FromStr;
use std::sync::Arc;
use anyhow::anyhow;
use serde_json::{Map, Value};
use wd_tools::PFErr;
use rush_core::{Exec, FunctionSet};
use crate::Calc;

#[derive(Debug,Default)]
pub struct Assign{
    execs:HashMap<String,Calc>
}
impl Assign{
    pub fn new()->Self{
        Assign{execs:HashMap::new()}
    }
    pub fn add_exec<K:Into<String>,C:Into<Calc>>(mut self,key:K,expr:C)->Self{
        self.execs.insert(key.into(),expr.into());self
    }
    #[allow(unused_assignments)]
    fn insert_value(k:&str, input:Value, mut out: &mut Value) ->anyhow::Result<()>{
        let ks:Vec<_> = k.split(".").collect();
        let last = ks.len() - 1;
        for (i,e) in ks.into_iter().enumerate(){
            if let Value::Object(map) = out{
                if i == last{
                    map.insert(e.to_string(),input);
                    return Ok(())
                }
                if map.get(e).is_none() {
                    map.insert(e.to_string(),Value::Object(Map::new()));
                }
                if let Some(s) = map.get_mut(e){
                    out = s;
                }
            }
            return anyhow!("want insert at:{},but the path is not obj:{}",k,input).err()
        }
        return Ok(())
    }
}
impl Exec for Assign{
    fn execute(&self, fs: Arc<dyn FunctionSet>, input: &Value, output: &mut Value) -> anyhow::Result<()> {
        for (k,c) in self.execs.iter(){
            let val = c.value(&fs,input)?;
            Self::insert_value(k,val,output)?;
        }
        Ok(())
    }
}

impl FromStr for Assign{
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = s.trim_start_matches(" \r\n\t");
        let ss:Vec<_> = s.split(";").collect();
        let mut assign = Assign::new();
        for i in ss{
            let expr = i.trim_matches(|x|" \r\n\t".contains(x));
            if expr.is_empty() {
                continue
            }
            if let Some((k,e)) = expr.split_once("="){
                assign = assign.add_exec(k.trim_matches(|x|" \r\n\t".contains(x)),e);
            }else {
                return anyhow!("parse[{}] failed, expr must format:[argument = expression]",i).err();
            }
        }
        Ok(assign)
    }
}

#[cfg(test)]
mod test{
    use rush_core::Exec;
    use crate::Assign;

    #[test]
    fn test_assign_new(){
        let exec_expression = r#"
        data.message = 'success';
        data.code = 0;
        data.value1 = [1,2,3];
        data.value2 = args1 + args2;
        data.value3 = !args3;
        data.value4 = str_len('hello world');
        data.value5 = 1>>2;
        "#;
        let a = exec_expression.parse::<Assign>().expect("new Assign failed");
        println!("--->{:?}",a);
    }
}