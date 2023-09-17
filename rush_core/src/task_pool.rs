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

// impl Rush{
//     pub async fn multi_flow<Obj: Serialize, Out:for<'de> Deserialize<'de>>(&self, obj: Obj) -> anyhow::Result<Out> {
//         let obj:Value = serde_json::to_value(obj)?;
//         let (send, mut recv) = mpsc::channel(self.nodes.len());
//         let obj = Arc::new(obj);
//         for (k, v) in self.nodes.iter() {
//             let rule_name = k.to_string();
//             let send = send.clone();
//             let fs = self.functions.share();
//             let obj = obj.clone();
//             let nodes = v.clone();
//             tokio::spawn(async move {
//                 for i in nodes.iter(){
//                     match i.when(fs.clone(),&obj) {
//                         Ok(b) => {
//                             if !b{
//                                 if let Err(e) = send.send(MulMsg::new(Ok(b),rule_name.clone())).await{
//                                     println!("rush.multi_flow false recv is close:{}",e);
//                                 }
//                                 return
//                             }
//                         }
//                         Err(e) => {
//                             if let Err(e) = send.send(MulMsg::new(Err(e),rule_name.clone())).await{
//                                 println!("rush.multi_flow  error recv is close:{}",e);
//                             }
//                             return
//                         }
//                     }
//                 }
//                 if let Err(e) = send.send(MulMsg::new(Ok(true),rule_name)).await{
//                     println!("rush.multi_flow over recv is close:{}",e);
//                 }
//             });
//         }
//         let mut rules = vec![];
//         for _ in 0..self.nodes.len(){
//             if let Some(i) = recv.recv().await{
//                 if i.result? {
//                     rules.push(i.rule_name);
//                 }
//             }else{
//                 println!("rush.multi_flow should is not null");
//                 break
//             }
//         }
//         let val = self.execute(&obj, rules)?;
//         let val = serde_json::from_value(val)?;Ok(val)
//     }
// }