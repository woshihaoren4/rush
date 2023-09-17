use std::future::Future;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::sync::Arc;

// 核心抽象，所有规则算法都可以看作是一个过滤器，给如一个输入，给出一个输出
pub trait Filter {
    fn flow<Obj: Serialize, Out: Deserialize<'static>>(&self, obj: Obj) -> anyhow::Result<Out>;
}
// 计算节点
pub trait CalcNode {
    fn when(&self, fs: Arc<dyn FunctionSet>, input: &Value) -> anyhow::Result<bool>;
}
// 运算规则
pub trait Exec {
    fn execute(
        &self,
        fs: Arc<dyn FunctionSet>,
        input: &Value,
        output: &mut Value,
    ) -> anyhow::Result<()>;
}
// 函数
pub trait Function  {
    fn call(&self, fs: Arc<dyn FunctionSet>, args: Vec<Value>) -> anyhow::Result<Value>;
}
// 函数集
pub trait FunctionSet {
    fn get(&self, name: &str) -> Option<Arc<dyn Function>>;
}

#[async_trait::async_trait]
pub trait TaskPool{
    type Out;
    async fn push<F:Future<Output=Self::Out>+Send+'static>(&self,task:F)->anyhow::Result<Self::Out>{
        Ok(task.await)
    }
}
