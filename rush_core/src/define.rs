use std::sync::Arc;
use serde::{Deserialize, Serialize};
use serde_json::Value;

// 核心抽象，所有规则算法都可以看作是一个过滤器，给如一个输入，给出一个输出
pub trait Filter{
    fn input<Obj:Serialize,Out:Deserialize<'static>>(&self, obj:Obj) ->anyhow::Result<Out>;
}
// 计算节点
pub trait CalcNode{
    fn when(&self,fs:Arc<dyn FunctionSet>, input:&Value)-> anyhow::Result<bool>;
}
// 运算规则
pub trait Rule{
    fn execute(&self,fs:Arc<dyn FunctionSet>,input:&Value,output:&mut Value)->anyhow::Result<()>;
}
// 函数
pub trait Function{
    fn call(&self,fs:Arc<dyn FunctionSet>,args:Vec<Value>)->anyhow::Result<Value>;
}
// 函数集
pub trait FunctionSet{
    fn get(&self,name:&str)->Option<Arc<dyn Function>>;
}