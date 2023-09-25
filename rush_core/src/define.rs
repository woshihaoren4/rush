use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::sync::Arc;

// 核心抽象，所有规则算法都可以看作是一个过滤器，给如一个输入，给出一个输出
pub trait RuleFlow {
    fn version(&self) -> i32 {
        1
    }
    fn flow<Obj: Serialize, Out: for<'a> Deserialize<'a>>(&self, obj: Obj) -> anyhow::Result<Out>;
}

#[async_trait::async_trait]
pub trait AsyncRuleFlow: RuleFlow + Sync + Send {
    async fn async_flow<Obj: Serialize + Send, Out: for<'a> Deserialize<'a>>(
        &self,
        obj: Obj,
    ) -> anyhow::Result<Out> {
        self.flow(obj)
    }
}

// 计算节点
pub trait CalcNode: Send + Sync {
    fn when(&self, fs: Arc<dyn FunctionSet>, input: &Value) -> anyhow::Result<bool>;
}
// 运算规则
pub trait Exec: Send + Sync {
    fn execute(
        &self,
        fs: Arc<dyn FunctionSet>,
        input: &Value,
        output: &mut Value,
    ) -> anyhow::Result<()>;
}
// 函数
pub trait Function: Send + Sync {
    fn call(&self, fs: Arc<dyn FunctionSet>, args: Vec<Value>) -> anyhow::Result<Value>;
}
// 函数集
pub trait FunctionSet: Send + Sync {
    fn get(&self, name: &str) -> Option<Arc<dyn Function>>;
}

#[async_trait::async_trait]
pub trait RuleEngineDiscovery<F> {
    fn version(&self) -> i32 {
        1
    }
    async fn upgrade(&self) -> F;
}
