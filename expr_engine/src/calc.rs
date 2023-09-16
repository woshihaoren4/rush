use crate::{CalcBuilder, NotFoundFieldError};
use anyhow::anyhow;
use rush_core::FunctionSet;
use serde_json::{Number, Value};
use std::fmt::Debug;
use std::str::FromStr;
use std::sync::Arc;
use wd_tools::{PFErr, PFOk};

#[derive(Debug, Eq, PartialEq, Clone)]
pub enum Opt {
    //最低
    ADD, // +
    SUB, // -
    MUL, // *
    DIV, // /
    REM, // %
    AND, // &
    OR,  // |
    XOR, // ^
    NOT, // !
    REV, // ~
    SHL, // <<
    SHR, // >>
    //次高
    GT, // >
    GE, // >=
    LT, // <
    LE, // <=
    EQ, // ==
    NQ, // !=
    // 优先级最高
    AT, // &&
    OT, // ||
}

impl AsRef<str> for Opt {
    fn as_ref(&self) -> &str {
        match self {
            Opt::ADD => "+",
            Opt::SUB => "-",
            Opt::MUL => "*",
            Opt::DIV => "/",
            Opt::REM => "%",
            Opt::AND => "&",
            Opt::OR => "|",
            Opt::XOR => "^",
            Opt::NOT => "!",
            Opt::REV => "~",
            Opt::SHL => "<<",
            Opt::SHR => ">>",
            Opt::GT => ">",
            Opt::GE => ">=",
            Opt::LT => "<",
            Opt::LE => "<=",
            Opt::EQ => "==",
            Opt::NQ => "!=",
            Opt::AT => "&&",
            Opt::OT => "||",
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum Calc {
    NULL,
    Field(String),
    String(String),
    Number(i64),
    Float(f64),
    Bool(bool),
    Array(Vec<Calc>),
    Function(String, Vec<Calc>),

    Operator(Opt, Vec<Calc>),
}

macro_rules! operator_number_float {
    ($operator:tt,$args:tt,$fs:tt,$input:tt,$($enum_ty:path=>$opt:tt),*) => {
        $(
        if let $enum_ty = $operator{
            let nb1 = $args[0].number($fs, $input)?;
            let nb2 = $args[1].number($fs,$input)?;
            if let Some(i1) = nb1.as_i64() {
                if let Some(i2) = nb2.as_i64(){
                    return  Value::Number(Number::from(i1 $opt i2)).ok()
                }else if let Some(i2) = nb2.as_f64(){
                    if let Some(s) = Number::from_f64(i1 as f64 $opt i2){
                        return Value::Number(s).ok()
                    }
                }
            }else if let Some(f1) = nb1.as_f64(){
                if let Some(f2) = nb2.as_i64(){
                    if let Some(s) = Number::from_f64(f1 $opt f2 as f64){
                        return Value::Number(s).ok()
                    }
                }else if let Some(f2) = nb2.as_f64(){
                    if let Some(s) = Number::from_f64(f1 $opt f2){
                        return Value::Number(s).ok()
                    }
                }
            }
            return anyhow!("operator[{:?}] can not support args:[{:?}]",$operator,$args).err()
        }
        )*
    };
}
macro_rules! operator_number_bool {
    ($operator:tt,$args:tt,$fs:tt,$input:tt,$($enum_ty:path=>$opt:tt),*) => {
        $(
        if let $enum_ty = $operator{
            let nb1 = $args[0].number($fs, $input)?;
            let nb2 = $args[1].number($fs,$input)?;
            if let Some(i1) = nb1.as_i64() {
                if let Some(i2) = nb2.as_i64(){
                    return  Value::Bool(i1 $opt i2).ok()
                }else if let Some(i2) = nb2.as_f64(){
                    return Value::Bool((i1 as f64) $opt i2).ok()
                }
            }else if let Some(f1) = nb1.as_f64(){
                if let Some(f2) = nb2.as_i64(){
                    return Value::Bool(f1 $opt (f2 as f64)).ok()
                }else if let Some(f2) = nb2.as_f64(){
                    return Value::Bool(f1 $opt f2).ok()
                }
            }
            return anyhow!("operator[{:?}] can not support args:[{:?}]",$operator,$args).err()
        }
        )*
    };
}
macro_rules! operator_number_bit_option {
    ($operator:tt,$args:tt,$fs:tt,$input:tt,$($enum_ty:path=>$opt:tt),*) => {
        $(
            if let $enum_ty = $operator{
            let nb1 = $args[0].number($fs, $input)?;
            let nb2 = $args[1].number($fs,$input)?;
            if let Some(i1) = nb1.as_i64() {
                if let Some(i2) = nb2.as_i64(){
                    return  Value::Number(Number::from(i1 $opt i2)).ok()
                }
            }
            return anyhow!("operator[{:?}] can not support args:[{:?}]",$operator,$args).err()
        }
        )*
    };
}

impl Calc {
    pub fn field(&self, mut input: &Value) -> anyhow::Result<Value> {
        match self {
            Calc::Field(field) => {
                let ks: Vec<&str> = field.split('.').collect();
                for i in ks {
                    match input {
                        Value::Object(obj) => {
                            match obj.get(i) {
                                None => {
                                    // return <NotFoundFieldError as Into<anyhow::Error>>::into(NotFoundFieldError(i.to_string())).err()
                                    return Err(NotFoundFieldError(i.to_string()).into());
                                }
                                Some(s) => {
                                    input = s;
                                }
                            }
                        }
                        _ => return anyhow!("not found object at field[{field}]").err(),
                    }
                }
                input.clone().ok()
            }
            _ => anyhow!("calc[{:?}],is not field", self).err(),
        }
    }
    pub fn function(&self, fs: &Arc<dyn FunctionSet>, input: &Value) -> anyhow::Result<Value> {
        return match self {
            Calc::Function(name, args) => {
                let mut val_args = vec![];
                for i in args {
                    val_args.push(i.value(fs, input)?);
                }
                if let Some(function) = fs.get(name) {
                    function.call(fs.clone(), val_args)
                } else {
                    anyhow!("function[{}] not found", name).err()
                }
            }
            _ => anyhow!("type[{:?}] is not function", self).err(),
        };
    }
    pub fn number(&self, fs: &Arc<dyn FunctionSet>, input: &Value) -> anyhow::Result<Number> {
        let n = match self {
            Calc::NULL => Number::from(0i64),
            Calc::Field(_) => {
                let val = self.field(input)?;
                match val {
                    Value::Null => Number::from(0i64),
                    Value::Number(n) => n,
                    _ => return anyhow!("type[{val}] can not to number").err(),
                }
            }
            Calc::Number(n) => Number::from(*n),
            Calc::Float(f) => {
                if let Some(s) = Number::from_f64(*f) {
                    s
                } else {
                    return anyhow!("want get f64,found NAN").err();
                }
            }
            Calc::Function(_, _) => {
                let val = self.function(fs, input)?;
                match val {
                    Value::Null => Number::from(0i64),
                    Value::Number(n) => n,
                    _ => return anyhow!("type[{val}] can not to number").err(),
                }
            }
            _ => return anyhow!("type[{:?}] can not to number", self).err(),
        };
        return n.ok();
    }
    pub fn value(&self, fs: &Arc<dyn FunctionSet>, input: &Value) -> anyhow::Result<Value> {
        let b = match self {
            Calc::NULL => Value::Null,
            Calc::Field(_) => self.field(input)?,
            Calc::String(s) => Value::String(s.clone()),
            Calc::Number(n) => Value::Number(Number::from(*n)),
            Calc::Float(f) => match Number::from_f64(*f) {
                None => return anyhow!("need f64, found a NAN").err(),
                Some(n) => Value::Number(n),
            },
            Calc::Bool(b) => Value::Bool(*b),
            Calc::Array(a) => {
                let mut array = vec![];
                for i in a {
                    array.push(i.value(fs, input)?);
                }
                Value::Array(array)
            }
            Calc::Function(_, _) => self.function(fs, input)?,
            Calc::Operator(opt, args) => Self::operator(opt, args, fs, input)?,
        };
        Ok(b)
    }
    pub fn operator(
        opt: &Opt,
        args: &Vec<Calc>,
        fs: &Arc<dyn FunctionSet>,
        input: &Value,
    ) -> anyhow::Result<Value> {
        if args.len() == 0 {
            return anyhow!("operator[{:?}] args count must have one", opt).err();
        }
        if args.len() == 1 {
            match opt {
                Opt::SUB => {
                    let n = args[0].number(fs, input)?;
                    if let Some(i) = n.as_i64() {
                        return Value::Number(Number::from(-i)).ok();
                    } else if let Some(i) = n.as_u64() {
                        return Value::Number(Number::from(-(i as i64))).ok();
                    }
                    if let Some(i) = n.as_f64() {
                        if let Some(s) = Number::from_f64(-i) {
                            return Value::Number(s).ok();
                        }
                    }
                    return anyhow!("operator '-' parse number[{n}] failed").err();
                }
                Opt::NOT => {
                    let b = args[0].bool(fs, input)?;
                    return Value::Bool(b).ok();
                }
                Opt::REV => {
                    let n = args[0].number(fs, input)?;
                    if let Some(i) = n.as_i64() {
                        return Value::Number(Number::from(!i)).ok();
                    }
                    return anyhow!("operator '~' only support i64 type").err();
                }
                _ => {}
            }
        }
        if args.len() != 2 {
            return anyhow!("operator[{:?}] args count must hava two", opt).err();
        }
        // let arg1 = args[0].value(fs,input)?;
        // let arg2 = args[1].value(fs,input)?;

        match opt {
            Opt::NOT | Opt::XOR => {
                return anyhow!("operator[{:?}] args count must is one", opt).err()
            }
            _ => {}
        }

        operator_number_float!(opt,args,fs,input,
            Opt::ADD=>+,
            Opt::SUB=>-,
            Opt::MUL=>*,
            Opt::DIV=>/,
            Opt::REM=>%);

        operator_number_bit_option!(opt,args,fs,input,
            Opt::AND=> &,
            Opt::OR=> |,
            Opt::XOR=> ^,
            Opt::SHL=> <<,
            Opt::SHR=> >>);

        operator_number_bool!(opt,args,fs,input,
            Opt::GT=> >,
            Opt::GE=> >=,
            Opt::LT=> <,
            Opt::LE=> <=);

        if let Opt::AT = opt {
            let b1 = args[0].bool(fs, input)?;
            let b2 = args[1].bool(fs, input)?;
            return Value::Bool(b1 && b2).ok();
        }
        if let Opt::OT = opt {
            if args[0].bool(fs, input)? {
                return Value::Bool(true).ok();
            }
            if args[1].bool(fs, input)? {
                return Value::Bool(true).ok();
            }
            return Value::Bool(false).ok();
        }

        let v1 = args[0].value(fs, input)?;
        let v2 = args[1].value(fs, input)?;
        if let Opt::EQ = opt {
            return Value::Bool(v1 == v2).ok();
        } else if let Opt::NQ = opt {
            return Value::Bool(v1 != v2).ok();
        }

        return anyhow!("unknown operator[{:?}]", opt).err();
    }

    pub fn bool(&self, fs: &Arc<dyn FunctionSet>, input: &Value) -> anyhow::Result<bool> {
        let b = match self {
            Calc::NULL => false,
            Calc::Field(_) => {
                let val = self.field(input)?;
                match val {
                    Value::Null => false,
                    Value::Bool(b) => b,
                    _ => true,
                }
            }
            Calc::String(s) => !s.is_empty(),
            Calc::Number(n) => *n != 0,
            Calc::Float(_) => true,
            Calc::Bool(b) => *b,
            Calc::Array(_) => true,
            Calc::Function(_, _) => {
                let val = self.function(fs, input)?;
                match val {
                    Value::Null => false,
                    Value::Bool(b) => b,
                    _ => true,
                }
            }
            Calc::Operator(opt, args) => {
                let val = Self::operator(opt, args, fs, input)?;
                match val {
                    Value::Null => false,
                    Value::Bool(b) => b,
                    _ => true,
                }
            }
        };
        Ok(b)
    }
}

impl ToString for Calc {
    fn to_string(&self) -> String {
        match self {
            Calc::NULL => "null".into(),
            Calc::Field(s) => s.clone(),
            Calc::String(s) => format!("\"{}\"", s),
            Calc::Number(n) => n.to_string(),
            Calc::Float(f) => format!("{:.2}", f),
            Calc::Bool(b) => b.to_string(),
            Calc::Array(list) => {
                let mut array: String = "[".into();
                for (i, e) in list.iter().enumerate() {
                    if i != 0 {
                        array.push_str(",");
                    }
                    array.push_str(e.to_string().as_str());
                }
                array.push_str("]");
                array
            }
            Calc::Function(func, args) => {
                let mut func = func.clone();
                func.push_str("(");
                for (i, arg) in args.iter().enumerate() {
                    if i != 0 {
                        func.push_str(",")
                    }
                    func.push_str(arg.to_string().as_str());
                }
                func.push_str(")");
                func
            }
            Calc::Operator(opt, arg) => {
                if arg.len() == 1 {
                    format!("({} {})", opt.as_ref(), arg[0].to_string())
                } else if arg.len() == 2 {
                    format!(
                        "({} {} {})",
                        arg[0].to_string(),
                        opt.as_ref(),
                        arg[1].to_string()
                    )
                } else {
                    panic!("Calc.to_string length[{}]", arg.len());
                }
            }
        }
    }
}

impl FromStr for Calc {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        CalcBuilder::new(s).build()
    }
}
impl<S: AsRef<str>> From<S> for Calc {
    fn from(value: S) -> Self {
        Calc::from_str(value.as_ref()).unwrap()
    }
}

impl rush_core::CalcNode for Calc {
    fn when(&self, fs: Arc<dyn FunctionSet>, input: &Value) -> anyhow::Result<bool> {
        match self.bool(&fs, input) {
            Ok(o) => o.ok(),
            Err(e) => {
                // not found field = false
                // no return error
                if let Some(_) = e.downcast_ref::<NotFoundFieldError>() {
                    Ok(false)
                } else {
                    e.err()
                }
            }
        }
    }
}

#[cfg(test)]
mod test {
    use super::Calc;
    use crate::Opt;
    use rush_core::{CalcNode, Function, FunctionSet};
    use serde::Serialize;
    use std::fmt::Debug;
    use std::sync::Arc;

    //cargo test --color=always --lib calc_impl::test::test_calc_show -- --exact unstable-options --nocapture
    #[test]
    fn test_calc_show() {
        let calc = Calc::Operator(
            Opt::GE,
            vec![
                Calc::Number(32),
                Calc::Function("str_len".into(), vec![Calc::String("hello world".into())]),
            ],
        );
        println!("result ===> {}", calc.to_string());
    }

    //cargo test --color=always --lib calc::test::test_calc_parse -- --exact unstable-options --nocapture
    #[test]
    fn test_calc_parse() {
        let calc: Calc = "a > b || c < d".parse().unwrap();
        println!("---> {}", calc.to_string());
    }

    #[derive(Serialize)]
    struct People {
        pub age: isize,
        pub native_place: String,
    }
    #[derive(Debug)]
    struct FunctionSetImpl;

    impl FunctionSet for FunctionSetImpl {
        fn get(&self, _name: &str) -> Option<Arc<dyn Function>> {
            None
        }
    }
    //cargo test --color=always --lib calc::test::test_calc_simple --no-fail-fast -- --exact unstable-options --nocapture
    #[test]
    fn test_calc_simple() {
        let expr = "age > 18 || native_place == \"中国\"";
        let calc: Calc = expr.parse().unwrap();
        println!("calc--->{}", calc.to_string());
        let people = People {
            age: 19,
            native_place: "japan".into(),
        };
        let input = serde_json::to_value(people).unwrap();
        let fs = Arc::new(FunctionSetImpl {});
        let result = calc.when(fs, &input).expect("when error===>");
        println!("result ---> {result}");
    }
}
