use std::collections::HashMap;
use std::fmt::{Debug, Formatter};
use std::sync::Arc;
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use crate::{CalcNode, Filter, Function, FunctionImpl, FunctionSet, HostFunction, Exec};
use wd_tools::sync::Acl;

pub struct Rush<C,R>{
    functions: Acl<HashMap<String,Arc<dyn Function>>>,
    nodes:HashMap<String,Vec<C>>,
    exec:HashMap<String,R>,
}

impl<C,R> Debug for Rush<C,R> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let mut fs = vec![];
        for (i,_) in self.functions.share().iter(){
            fs.push(i.to_string());
        }
        let mut nodes = vec![];
        for (i,_) in self.nodes.iter(){
            nodes.push(i.to_string());
        }
        let mut rules = vec![];
        for (i,_) in self.exec.iter(){
            rules.push(i.to_string());
        }
        write!(f, "{{ functions:{:?},nodes:{:?},rules:{:?} }}", fs,nodes,rules)
    }
}

impl<C:CalcNode, E: Exec> Rush<C, E> {
    pub fn new()->Self{
        let functions = Acl::new(HashMap::new());
        let nodes = HashMap::new();
        let rules = HashMap::new();
        Self{functions,nodes, exec: rules }
    }
    pub fn register_rule<T:Into<String>>(mut self, name:T, nodes:Vec<C>, exec: E) ->Self{
        let name = name.into();
        self.nodes.insert(name.clone(),nodes);
        self.exec.insert(name, exec);
        self
    }
    pub fn delete_rule<T:AsRef<str>>(&mut self,name:T){
        self.nodes.remove(name.as_ref());
        self.exec.remove(name.as_ref());
    }
    pub fn raw_register_function<S:Into<String>,F:Function>(self,name:S,function:F)->Self{
        self.functions.update(|x|{
            let mut map = (*x).clone();
            map.insert(name.into(),Arc::new(function));
            map
        });self
    }
    pub fn register_function<S:Into<String>,Args,Out,F>(self,name:S,function:F)->Self
    where F:HostFunction<Args,Out> + 'static,Out:Serialize,
    {
        self.raw_register_function(name,FunctionImpl::new(function))
    }

    pub fn delete_function<S:AsRef<str>>(self,name:S)->Self{
        self.functions.update(|x|{
            let mut map = (*x).clone();
            map.remove(name.as_ref());
            map
        });self
    }

    /// input_value
    /// 1. 计算匹配到的规则
    /// 2. 找出规则进行结果生成
    fn input_value(&self,obj:Value)->anyhow::Result<Value>{
        let mut rules = vec![];
        'lp : for (k,v) in self.nodes.iter(){
            for i in v.iter(){
                if !i.when(self.functions.share(),&obj)?{
                    continue 'lp
                }
            }
            rules.push(k.to_string());
        }

        let mut output = Value::Object(Map::new());
        for name in rules.iter(){
            if let Some(r) = self.exec.get(name){
                r.execute(self.functions.share(),&obj,&mut output)?;
            }
        }
        Ok(output)
    }
}

impl<C, E,I:IntoIterator<Item=(String, Vec<C>, E)>> From<I> for Rush<C, E>
where C:CalcNode, E: Exec
{
    fn from(value: I) -> Self {
        let mut rush = Rush::new();
        for (name,calc,exec) in value{
            rush = rush.register_rule(name,calc,exec);
        }
        rush
    }
}

impl FunctionSet for HashMap<String,Arc<dyn Function>>  {
    fn get(&self, name: &str) -> Option<Arc<dyn Function>> {
        self.get(name).map(|a|a.clone())
    }
}

impl<C:CalcNode,R: Exec> Filter for Rush<C,R>  {
    fn input<Obj: Serialize, Out: Deserialize<'static>>(&self, obj: Obj) -> anyhow::Result<Out> {
        let value = serde_json::to_value(obj)?;
        let result = self.input_value(value)?;
        let out = Out::deserialize(result)?;Ok(out)
    }
}

#[cfg(test)]
mod test{
    use std::sync::Arc;
    use serde::{Deserialize, Serialize};
    use serde_json::Value;
    use crate::{CalcNode, Filter, FunctionSet, Exec};
    use crate::rush::Rush;

    struct CalcNodeImpl;
    impl CalcNode for CalcNodeImpl{
        fn when(&self,_fs:Arc<dyn FunctionSet>, _input: &Value) ->anyhow::Result<bool>{
            return Ok(true)
        }
    }
    struct RuleImpl;
    impl Exec for RuleImpl{
        fn execute(&self,_fs:Arc<dyn FunctionSet>,_input:&Value,_output:&mut Value)->anyhow::Result<()> {
            Ok(())
        }
    }
    #[derive(Debug,Default,Serialize,Deserialize)]
    struct ObjTest{
        #[serde(default="String::default")]
        pub name:String
    }

    //cargo test --color=always --lib rush::test::test_simple --no-fail-fast -- --exact unstable-options --show-output
    #[test]
    fn test_simple(){
        let mr = Rush::<CalcNodeImpl,RuleImpl>::new();
        let result:ObjTest = mr.input(ObjTest { name: "hello world".into()}).expect("input failed");
        println!("result ---> {result:?}");
    }
}