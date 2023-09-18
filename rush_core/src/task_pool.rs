use std::sync::Arc;
use serde::{Deserialize, Serialize};
use serde_json::{Value};
use crate::Rush;
use tokio::sync::mpsc;
#[derive(Debug)]
pub struct MulMsg{
    pub result:anyhow::Result<bool>,
    pub rule_name:String,
}
impl MulMsg{
    pub fn new(result:anyhow::Result<bool>,rule_name:String)->Self{
        Self{result,rule_name}
    }
}

#[derive(Debug)]
pub struct MultiRush{
    rush:Arc<Rush>
}
impl MultiRush{
    pub async fn multi_flow<Obj: Serialize, Out:for<'de> Deserialize<'de>>(&self, obj: Obj) -> anyhow::Result<Out> {
        let obj:Value = serde_json::to_value(obj)?;
        let (send, mut recv) = mpsc::channel(self.rush.nodes.len());
        let obj = Arc::new(obj);
        for (k, _) in self.rush.nodes.iter() {
            let rh = self.rush.clone();
            let rule_name = k.to_string();
            let obj = obj.clone();
            let send = send.clone();
            tokio::spawn(async move {
                let cs = if let Some(i) = rh.nodes.get(rule_name.as_str()){
                    i
                }else{
                    return //not to here
                };
                for i in cs.iter(){
                    match i.when(rh.functions.share(),&obj) {
                        Ok(b) => {
                            if !b{
                                if let Err(e) = send.send(MulMsg::new(Ok(b),rule_name.clone())).await{
                                    println!("rush.multi_flow false recv is close:{}",e);
                                }
                                return
                            }
                        }
                        Err(e) => {
                            if let Err(e) = send.send(MulMsg::new(Err(e),rule_name.clone())).await{
                                println!("rush.multi_flow  error recv is close:{}",e);
                            }
                            return
                        }
                    }
                }
                if let Err(e) = send.send(MulMsg::new(Ok(true),rule_name)).await{
                    println!("rush.multi_flow over recv is close:{}",e);
                }
            });
        }
        let mut rules = vec![];
        for _ in 0..self.rush.nodes.len(){
            if let Some(i) = recv.recv().await{
                if i.result? {
                    rules.push(i.rule_name);
                }
            }else{
                println!("rush.multi_flow should is not null");
                break
            }
        }
        drop(send);
        let val = self.rush.execute(&obj, rules)?;
        let val = serde_json::from_value(val)?;Ok(val)
    }
}

impl From<Rush> for MultiRush{
    fn from(value: Rush) -> Self {
        Self{rush:Arc::new(value)}
    }
}