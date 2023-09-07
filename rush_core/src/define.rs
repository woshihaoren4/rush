use serde::{Deserialize, Serialize};
use serde_json::Value;

// 核心抽象，所有规则算法都可以看作是一个过滤器，给如一个输入，给出一个输出
pub trait Filter{
    fn input<Obj:Serialize,Out:Deserialize<'static>>(&self, obj:Obj) ->anyhow::Result<Out>;
}
// 计算节点
pub trait CalcNode{
    fn when(&self,input:&Value)->bool;
}
// 运算规则
pub trait Rule{
    fn execute(&self,input:&Value,output:&mut Value);
}