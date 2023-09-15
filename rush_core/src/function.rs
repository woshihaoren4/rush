use std::sync::Arc;
use anyhow::anyhow;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use wd_tools::{PFErr, PFOk};
use crate::{Function, FunctionSet};

pub trait FromValue: Sized{
    fn from(val:Value)->anyhow::Result<Self> ;
}
// impl FromValue for String{
//     fn from(value: Value)->anyhow::Result<Self>{
//         if let Value::String(s) = value{
//             return  s.ok()
//         }
//         return anyhow!("unknown type[{}] FromValue for String",value).err()
//     }
// }
// macro_rules! from_value_i64 {
//     ($($t:ty),*) => {
//         $(
//         impl FromValue for $t{
//             fn from(val: Value) -> anyhow::Result<Self> {
//             if let Value::Number(ref n) = val{
//                 if let Some(i) = n.as_i64(){
//                     return (i as $t).ok()
//                 }
//             }
//             return anyhow!("unknown type[{}] FromValue for number",val).err()
//             }
//         }
//         )*
//     };
// }
// from_value_i64!(i8,i16,i32,i64,isize,u8,u16,u32,u64,usize);
//
// macro_rules! from_value_f64 {
//     ($($t:ty),*) => {
//         $(
// impl FromValue for $t{
//     fn from(val: Value) -> anyhow::Result<Self> {
//     if let Value::Number(ref n) = val{
//         if let Some(f) = n.as_f64(){
//             return (f as $t).ok()
//         }
//     }
//     return anyhow!("unknown type[{}] FromValue for float",val).err()
// }
// }
//         )*
//     };
// }
// from_value_f64!(f32,f64);
//
// impl FromValue for bool{
//     fn from(val: Value) -> anyhow::Result<Self> {
//         if let Value::Bool(b) = val{
//             return b.ok()
//         }
//         return anyhow!("unknown type[{}] FromValue for bool",val).err()
//     }
// }
// impl FromValue for Option<()>{
//     fn from(val: Value) -> anyhow::Result<Self> {
//         if let Value::Null = val{
//             return None.ok()
//         }
//         return anyhow!("unknown type[{}] FromValue for null",val).err()
//     }
// }
//
// impl<T:FromValue> FromValue for Vec<T>{
//     fn from(val: Value) -> anyhow::Result<Self> {
//         if let Value::Array(array) = val{
//             let mut list = vec![];
//             for i in array{
//                 list.push(T::from(i)?);
//             }
//             return list.ok()
//         }
//         return anyhow!("unknown type[{}] FromValue for array",val).err()
//     }
// }
impl<T> FromValue for T
where T: for<'a> Deserialize<'a>
{
    fn from(val: Value) -> anyhow::Result<Self> {
        let t:T = serde_json::from_value(val)?;t.ok()
    }
}


pub trait HostFunction<A,O>{
    fn call(&self,args:Vec<Value>)->anyhow::Result<Value>;
}
pub struct FunctionImpl<A,O>{
    inner:Box<dyn HostFunction<A,O>>
}

impl<A,O> FunctionImpl<A,O>{
    pub fn new<F:HostFunction<A,O>+ 'static>(f:F)->Self{
        let inner = Box::new(f);
        Self{inner}
    }
}

impl<A,O> Function for FunctionImpl<A,O>
where O:Serialize
{
    fn call(&self, _fs: Arc<dyn FunctionSet>, args: Vec<Value>) -> anyhow::Result<Value> {
        self.inner.call(args)
    }
}
impl<O,F> HostFunction<(),O> for F
where O:Serialize,F:Fn()->anyhow::Result<O> + 'static
{
    fn call(&self, _args: Vec<Value>) -> anyhow::Result<Value> {
        let out = self()?;
        let val = serde_json::to_value(out)?;
        Ok(val)
    }
}
macro_rules! function_impl_template {
    ($n:tt,$($t:tt),*) => {
        impl<$($t,)* O,F> HostFunction<($($t,)*),O> for F
where $($t:FromValue,)*
        O:Serialize,F:Fn($($t,)*)->anyhow::Result<O> + 'static
{
    fn call(&self, mut args: Vec<Value>) -> anyhow::Result<Value> {
        if args.len() < $n {
            return anyhow!("expecting {} parameters actually finds {} parameters",$n,args.len()).err()
        }
        let out = self($($t::from(args.remove(0))?,)*)?;
        let val = serde_json::to_value(out)?;
        Ok(val)
    }
}
    };
}
function_impl_template!(1,A1);
function_impl_template!(2,A1,A2);
function_impl_template!(3,A1,A2,A3);
function_impl_template!(4,A1,A2,A3,A4);
function_impl_template!(5,A1,A2,A3,A4,A5);
function_impl_template!(6,A1,A2,A3,A4,A5,A6);
function_impl_template!(7,A1,A2,A3,A4,A5,A6,A7);
function_impl_template!(8,A1,A2,A3,A4,A5,A6,A7,A8);
function_impl_template!(9,A1,A2,A3,A4,A5,A6,A7,A8,A9);
function_impl_template!(10,A1,A2,A3,A4,A5,A6,A7,A8,A9,A10);
function_impl_template!(11,A1,A2,A3,A4,A5,A6,A7,A8,A9,A10,A11);
function_impl_template!(12,A1,A2,A3,A4,A5,A6,A7,A8,A9,A10,A11,A12);
function_impl_template!(13,A1,A2,A3,A4,A5,A6,A7,A8,A9,A10,A11,A12,A13);
function_impl_template!(14,A1,A2,A3,A4,A5,A6,A7,A8,A9,A10,A11,A12,A13,A14);
function_impl_template!(15,A1,A2,A3,A4,A5,A6,A7,A8,A9,A10,A11,A12,A13,A14,A15);
function_impl_template!(16,A1,A2,A3,A4,A5,A6,A7,A8,A9,A10,A11,A12,A13,A14,A15,A16);

#[cfg(test)]
mod test{
    use std::sync::Arc;
    use serde::Serialize;
    use serde_json::Value;
    use crate::{Function, FunctionImpl, FunctionSet, HostFunction};
    #[derive(Debug)]
    struct FSet;
    impl FunctionSet for FSet{
        fn get(&self, _name: &str) -> Option<Arc<dyn Function>> {
            None
        }
    }

    fn call<A,O:Serialize,F:HostFunction<A,O> + 'static>(f:F){
        let f = FunctionImpl{inner:Box::new(f)};
        let b :Box<dyn Function> = Box::new(f);
        let _ = b.call(Arc::new(FSet{}),vec![Value::String("hello".into()),Value::String("world".into())]).unwrap();
        // let _ =  f.call(Arc::new(FSet{}),vec![Value::String("hello".into()),Value::String("world".into())]);
    }

    //cargo test --color=always --lib function::test::test_fn --no-fail-fast -- --exact unstable-options --show-output --nocapture
    #[test]
    fn test_fn(){
        call(|a:String|{
            println!("--->{}",a);
            Ok("hello")
        });
    }
}