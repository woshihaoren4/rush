use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use crate::{CalcNode, Filter, Rule};

#[derive(Debug)]
pub struct MultipleRush<C,R>{
    nodes:HashMap<String,Vec<C>>,
    rules:HashMap<String,R>,
}

impl<C:CalcNode,R:Rule> MultipleRush<C,R> {
    pub fn new()->Self{
        let nodes = HashMap::new();
        let rules = HashMap::new();
        Self{nodes,rules}
    }
    pub fn register_rule<T:ToString>(mut self,name:T,nodes:Vec<C>,rule:R)->Self{
        self.nodes.insert(name.to_string(),nodes);
        self.rules.insert(name.to_string(),rule);
        self
    }
    fn input_value(&self,obj:Value)->anyhow::Result<Value>{
        let mut rules = vec![];
        'lp : for (k,v) in self.nodes.iter(){
            for i in v.iter(){
                if !i.when(&obj){
                    continue 'lp
                }
            }
            rules.push(k.to_string());
        }

        let mut output = Value::Object(Map::new());
        for name in rules.iter(){
            if let Some(r) = self.rules.get(name){
                r.execute(&obj,&mut output);
            }
        }
        Ok(output)
    }
}

impl<C:CalcNode,R:Rule> Filter for MultipleRush<C,R>  {
    fn input<Obj: Serialize, Out: Deserialize<'static>>(&self, obj: Obj) -> anyhow::Result<Out> {
        let value = serde_json::to_value(obj)?;
        let result = self.input_value(value)?;
        let out = Out::deserialize(result)?;Ok(out)
    }
}

#[cfg(test)]
mod test{
    use serde::{Deserialize, Serialize};
    use serde_json::Value;
    use crate::{CalcNode, Filter, Rule};
    use crate::multiple_rush::MultipleRush;

    struct CalcNodeImpl;
    impl CalcNode for CalcNodeImpl{
        fn when(&self, _input: &Value) -> bool {
            return true
        }
    }
    struct RuleImpl;
    impl Rule for RuleImpl{
        fn execute(&self,input:&Value,output:&mut Value) {
        }
    }
    #[derive(Debug,Default,Serialize,Deserialize)]
    struct ObjTest{
        #[serde(default="String::default")]
        pub name:String
    }

    //cargo test --color=always --lib multiple_rush::test::test_simple --no-fail-fast -- --exact unstable-options --show-output
    #[test]
    fn test_simple(){
        let mr = MultipleRush::<CalcNodeImpl,RuleImpl>::new();
        let result:ObjTest = mr.input(ObjTest { name: "hello world".into()}).expect("input failed");
        println!("result ---> {result:?}");
    }
}