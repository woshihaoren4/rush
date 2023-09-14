use std::cmp::Ordering;
use std::collections::VecDeque;
use std::str::FromStr;
use anyhow::anyhow;
use wd_tools::PFErr;
use crate::{Calc, Opt};

#[derive(Debug,Clone,PartialEq)]
pub enum Element{
    OPT(Opt),
    CALC(Calc),
    LeftSmall, // (
    RightSmall, // )
    LeftMed, // [
    RightMed, // ]
    LeftBig, // {
    RightBig, // }
    Comma, // ,
}

impl ToString for Element {
    fn to_string(&self) -> String {
        match self {
            Element::OPT(opt) => opt.as_ref().to_string(),
            Element::CALC(calc) => calc.to_string(),
            Element::LeftSmall => "(".into(),
            Element::RightSmall => ")".into(),
            Element::LeftMed => "[".into(),
            Element::RightMed => "]".into(),
            Element::LeftBig => "{".into(),
            Element::RightBig => "}".into(),
            Element::Comma => ",".into(),
        }
    }
}

macro_rules! match_one {
    ($o:expr,$e:expr,$deq:tt,$($key:tt=>$value:expr),*) => {
        $(
            if $e.starts_with($key) {
                $deq.push_back($value);
                $e = $e.split_off($key.len());
                $o
            }
        )*

    };
}
macro_rules! match_one_return {
    ($e:expr,$deq:tt,$($key:tt=>$value:expr),*) =>{
        match_one!(return true,$e,$deq,$($key => $value),*);
    }
}
#[allow(unused_macros)]
macro_rules! match_one_break {
    ($e:expr,$deq:tt,$($key:tt=>$value:expr),*) =>{
        match_one!(break,$e,$deq,$($key => $value),*);
    }
}

impl Opt{
    fn parse_one(expr:&mut String,deq:&mut VecDeque<Element>)->bool{
        match_one_return!(*expr,deq,
            //先解双字符
            "&&"=>Element::OPT(Opt::AT),
            "||"=>Element::OPT(Opt::OT),
            ">="=>Element::OPT(Opt::GE),
            "<="=>Element::OPT(Opt::LE),
            "=="=>Element::OPT(Opt::EQ),
            "!="=>Element::OPT(Opt::NQ),
            "<<"=>Element::OPT(Opt::SHL),
            ">>"=>Element::OPT(Opt::SHR),

            "<"=>Element::OPT(Opt::LT),
            ">"=>Element::OPT(Opt::GT),
            //低
            "+"=>Element::OPT(Opt::ADD),
            "-"=>Element::OPT(Opt::SUB),
            "*"=>Element::OPT(Opt::MUL),
            "/"=>Element::OPT(Opt::DIV),
            "%"=>Element::OPT(Opt::REM),
            "&"=>Element::OPT(Opt::AND),
            "|"=>Element::OPT(Opt::OR),
            "^"=>Element::OPT(Opt::XOR),
            "!"=>Element::OPT(Opt::NOT),
            "~"=>Element::OPT(Opt::REV));
        return false
    }
}


impl PartialOrd for Opt{
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {

        return match self {
            Opt::GT | Opt::GE | Opt::LT | Opt::LE | Opt::EQ | Opt::NQ  => {
                match other {
                    Opt::GT | Opt::GE | Opt::LT | Opt::LE | Opt::EQ | Opt::NQ  => {
                        Some(Ordering::Equal)
                    }
                    Opt::OT | Opt::AT => {
                        Some(Ordering::Less)
                    }
                    _  => {
                        Some(Ordering::Greater)
                    }
                }
            }
            Opt::OT | Opt::AT => {
                match other {
                    Opt::OT | Opt::AT => {
                        Some(Ordering::Equal)
                    }
                    Opt::GT | Opt::GE | Opt::LT | Opt::LE | Opt::EQ | Opt::NQ  => {
                        Some(Ordering::Greater)
                    }
                    _  => {
                        Some(Ordering::Greater)
                    }
                }
            }
            _  => {
                match other {
                    Opt::GT | Opt::GE | Opt::LT | Opt::LE | Opt::EQ | Opt::NQ  => {
                        Some(Ordering::Less)
                    }
                    Opt::OT | Opt::AT => {
                        Some(Ordering::Less)
                    }
                    _ => {
                        Some(Ordering::Equal)
                    }
                }
            }
        }
    }
}

impl Calc{
    //去掉注释
    pub(crate) fn parse_remove_annotation(mut s:String)->anyhow::Result<String> {
        let mut postion = vec![];
        let ss = s.as_bytes();
        let mut i = 0;
        while i < s.len()-1{
            if ss[i] == '/' as u8 && ss[i+1] == '*' as u8 { //左边界
                if postion.len() % 2 != 0{
                    return anyhow::anyhow!("'/*''*/'mismatch").err()
                }
                postion.push(i);
                i += 1;
            }else if ss[i] == '*' as u8 && ss[i+1] == '/' as u8{ //右边界
                if postion.len() % 2 != 1{
                    return anyhow::anyhow!("'/*''*/'mismatch").err()
                }
                postion.push(i+1);
                i += 1;
            }
            i += 1;
        }
        if postion.len() <= 0 {
            return Ok(s)
        }
        if postion.len() % 2 != 0{
            return anyhow::anyhow!("'/*''*/'mismatch").err()
        }
        let mut i = postion.len() as i32 - 1;
        while i > 0 {
            s.replace_range(postion[i as usize -1]..postion[i as usize]+1,"");
            i -= 2
        }
        return Ok(s)
    }
    //拆分
    pub(crate) fn expression_split(mut expr:String)->anyhow::Result<VecDeque<Element>>{
        let mut deq = VecDeque::new();
        'lp: while !expr.is_empty() {
            if expr.starts_with("\n"){
                expr = expr.split_off(1);
            }else if expr.starts_with("\r"){
                expr = expr.split_off(1);
            }else if expr.starts_with("\t"){
                expr = expr.split_off(1);
            }else if expr.starts_with(","){
                deq.push_back(Element::Comma);
                expr = expr.split_off(1);
            }else if expr.starts_with(" "){
                expr = expr.split_off(1);
            }else if expr.starts_with("(") {
                deq.push_back(Element::LeftSmall);
                expr = expr.split_off(1);
            }else if expr.starts_with(")"){
                deq.push_back(Element::RightSmall);
                expr = expr.split_off(1);
            }else if expr.starts_with("["){
                deq.push_back(Element::LeftMed);
                expr = expr.split_off(1);
            }else if expr.starts_with("]") {
                deq.push_back(Element::RightMed);
                expr = expr.split_off(1);
            }else if expr.starts_with("{"){
                deq.push_back(Element::LeftBig);
                expr = expr.split_off(1);
            }else if expr.starts_with("}"){
                deq.push_back(Element::RightBig);
                expr = expr.split_off(1);
            }else if expr.starts_with("\""){ //字符串
                let mut index = 0;
                for (i,e) in expr.as_bytes().into_iter().enumerate(){
                    if i != 0 && *e == '"' as u8{
                        index = i;break
                    }
                }
                let mut e = expr.split_off(index + 1);
                unsafe {
                    std::ptr::swap(&mut e,&mut expr);
                }
                let e = e.trim_matches(|c| c == '"');
                deq.push_back(Element::CALC(Calc::String(e.to_string())));
            }else if expr.starts_with("/*"){ //注释
                let mut index = 0;
                let bs = expr.as_bytes();
                while index < bs.len() - 1 {
                    if bs[index] == b'*' && bs[index +1] == b'/' {
                        expr = expr.split_off(index + 2);
                        continue 'lp
                    }
                    index+=1;
                }
                return anyhow!("/* and */ mismatch").err();
            }else if Opt::parse_one(&mut expr,&mut deq) {
                //运算符
            }else {
                let mut chars = expr.chars();
                let char = match chars.next(){
                    None => return anyhow!("parse error over").err(),
                    Some(s) => s,
                };
                if char.is_ascii_digit(){ //数字
                    let mut i = 1;
                    let mut not_is_float = true;
                    for e in chars.into_iter(){
                        if e == '.'{
                            not_is_float = false;
                        }else if !e.is_ascii_digit(){
                            break
                        }
                        i += 1;
                    }
                    let mut e = expr.split_off(i);
                    unsafe {
                        std::ptr::swap(&mut e,&mut expr);
                    }
                    if not_is_float {
                        let i = i64::from_str(e.as_str())?;
                        deq.push_back(Element::CALC(Calc::Number(i)));
                    }else{
                        let i = f64::from_str(e.as_str())?;
                        deq.push_back(Element::CALC(Calc::Float(i)));
                    }
                }else if char == '_' { //保留的部分
                    return anyhow!("Content beginning with the character _ is reserved").err();
                }else if char.is_alphabetic(){ //变量 或者函数 或者数组
                    let mut i = 1;
                    let mut ty = 1; //1:变量 2:函数
                    for e in chars.into_iter(){
                        if e == '(' { //函数
                            ty = 2;
                            break
                        }else if e.is_ascii_digit() || e.is_alphabetic() || e == '_' || e == '_'{

                        }else {
                            break
                        }
                        i+=1
                    }
                    let mut e = expr.split_off(i);
                    unsafe {
                        std::ptr::swap(&mut e,&mut expr);
                    }
                    if e == "true" || e == "false" {
                        deq.push_back(Element::CALC(Calc::Bool(bool::from_str(e.as_str()).unwrap())));
                    }else if e.to_lowercase() == "null" || e.to_lowercase() == "nil"{
                        deq.push_back(Element::CALC(Calc::NULL));
                    }else {
                        match ty {
                            1 => deq.push_back(Element::CALC(Calc::Field(e))),
                            2 => deq.push_back(Element::CALC(Calc::Function(e,vec![]))),
                            _ => return anyhow!("unknown string({e})").err(),
                        }
                    }

                }else{
                    return anyhow!("unknown char({char})").err();
                }
            }
        }
        return Ok(deq);
    }
    //组合

    // 取一个单元算子
    pub(crate) fn convert_one_group_calc(calc:Option<Calc>, deq:&mut VecDeque<Element>) -> anyhow::Result<Calc>{
        if deq.is_empty() {
            return if let Some(s) = calc {
                Ok(s)
            } else {
                anyhow!("expression is null").err()
            }
        };
        let ele = deq.pop_front().unwrap();
        match ele {
            Element::OPT(opt) => {
                match opt {
                    Opt::ADD | Opt::MUL | Opt::DIV | Opt::REM | Opt::AND | Opt::OR | Opt::XOR | Opt::SHL | Opt::SHR  => {
                        //向右边取一个算子单元
                        let lc = if let Some(s) = calc{s }else{
                            return anyhow!("parsing the left side of the operation[{opt:?}] failed").err()
                        };
                        let mut r_deq = Self::split_deque_by_opt(deq,&opt)?;
                        let rc = Self::convert_one_group_calc(None, &mut r_deq)?;
                        return Self::convert_one_group_calc(Some(Calc::Operator(opt,vec![lc, rc])), deq);
                    }
                    Opt::SUB | Opt::NOT => { // -
                        //向右边取一个算子单元
                        let mut r_deq = Self::split_deque_by_opt(deq,&opt)?;
                        let rc = Self::convert_one_group_calc(None, &mut r_deq)?;
                        return match calc {
                            None => Self::convert_one_group_calc(Some(Calc::Operator(opt, vec![rc])), deq),
                            Some(lc) => Self::convert_one_group_calc(Some(Calc::Operator(opt, vec![lc, rc])), deq),
                        }
                    }
                    Opt::REV => { // ~
                        if let Some(_) = calc{
                            return anyhow!("operation[{opt:?}] cannot perform multi-value calculations").err()
                        };
                        let mut r_deq = Self::split_deque_by_opt(deq,&opt)?;
                        let rc = Self::convert_one_group_calc(None, &mut r_deq)?;
                        return Self::convert_one_group_calc(Some(Calc::Operator(opt, vec![rc])), deq);
                    }
                    Opt::GT | Opt::GE | Opt::LT | Opt::LE | Opt::EQ | Opt::NQ | Opt::AT | Opt::OT => {
                        let lc = if let Some(s) = calc{s }else{
                            return anyhow!("parsing the left side of the operation[{opt:?}] failed").err()
                        };
                        let mut r_deq = Self::split_deque_by_opt(deq,&opt)?;
                        let rc = Self::convert_one_group_calc(None, &mut r_deq)?;
                        return Self::convert_one_group_calc(Some(Calc::Operator(opt, vec![lc,rc])), deq);
                    }
                }
            }
            Element::CALC(ec) => {
                return match ec {
                    Calc::NULL | Calc::Field(_) | Calc::String(_) | Calc::Number(_) | Calc::Float(_) | Calc::Bool(_) | Calc::Operator(_, _) | Calc::Array(_) => {
                        Self::convert_one_group_calc(Some(ec), deq)
                    }
                    Calc::Function(name, mut args) => {
                        if calc.is_some() {
                            return anyhow!("function[{name}] parse failed").err()
                        }
                        if let Some(Element::LeftSmall) = deq.pop_front(){

                        }else{
                            return anyhow!("function[{name}] right must is '('").err()
                        }
                        let list = Self::split_deque_by_comma(deq, Element::RightSmall)?;
                        for mut i in list {
                            let c = Self::convert_one_group_calc(None, &mut i)?;
                            args.push(c);
                        }
                        Self::convert_one_group_calc(Some(Calc::Function(name, args)), deq)
                    }
                }
            }
            Element::LeftSmall => {
                if calc.is_some() {
                    return anyhow!("'(' and ')' must bilateral symmetry").err()
                }
                // 拆分出一个单元节点
                let mut l_deq = Self::split_deque_by_ele(deq,Element::LeftSmall,Element::RightSmall)?;
                let lc = Self::convert_one_group_calc(None,&mut l_deq)?;
                return Self::convert_one_group_calc(Some(lc),deq);
            }
            Element::RightSmall => {
                return anyhow!("')' must match a '('").err()
            }
            Element::LeftMed => { //数组
                if calc.is_some() {
                    return anyhow!("'[' and ']' mut bilateral symmetry").err()
                }
                let list = Self::split_deque_by_comma(deq, Element::RightMed)?;
                let mut array = vec![];
                for mut i in list {
                    let c = Self::convert_one_group_calc(None, &mut i)?;
                    array.push(c);
                }
                return Self::convert_one_group_calc(Some(Calc::Array(array)), deq)
            }
            Element::RightMed => {
                return anyhow!("']' must match a '['").err()
            }
            Element::LeftBig => {
                return anyhow!("reserved character '{{'").err()
            }
            Element::RightBig => {
                return anyhow!("reserved character '}}'").err()
            }
            Element::Comma => {
                return anyhow!("You should use multiple operators instead of multiple expressions").err()
            }
        }
        // return Ok(Calc::NULL)
    }
    pub(crate) fn split_deque_by_comma(deq:&mut VecDeque<Element>,le:Element)->anyhow::Result<Vec<VecDeque<Element>>>{
        let mut sub_deq = VecDeque::new();
        let mut deq_list = vec![];
        let (mut count_small,mut count_med) = match le {
            Element::RightSmall=> (1,0),
            Element::RightMed=>(0,1),
            _=> return anyhow!("Calc.split_deque_by_comma nonsupport {:?}",le).err()
        };
        while let Some(e) = deq.pop_front() {
            let comma = match le {
                Element::RightSmall=> e == Element::Comma && count_med == 0 && count_small == 1,
                Element::RightMed=> e == Element::Comma && count_small == 0 && count_med == 1,
                _=> panic!(""),
            };
            if comma {
                deq_list.push(sub_deq.clone());
                sub_deq.clear();
                continue
            }
            if Element::LeftSmall == e {
                count_small += 1;
            }else if  Element::RightSmall == e{
                count_small -= 1;
            }else if Element::LeftMed == e{
                count_med += 1;
            }else if Element::RightMed == e{
                count_med -= 1;
            }
            if le == e && count_small == 0 && count_med == 0 {
                break
            }
            sub_deq.push_back(e);
        }
        if !sub_deq.is_empty() {
            deq_list.push(sub_deq.clone());
        }
        return Ok(deq_list)
    }
    pub(crate) fn split_deque_by_opt(deq:&mut VecDeque<Element>,opt:&Opt)->anyhow::Result<VecDeque<Element>> {
        let mut sub_deq = VecDeque::new();
        let mut count = 0i32;
        while let Some(e) = deq.pop_front() {
            if let Element::OPT(o) = e.clone(){
                if !(&o < opt) && count == 0 {
                    deq.push_front(Element::OPT(o));
                    break
                }
            }else if Element::LeftSmall == e {
                count += 1;
            }else if  Element::RightSmall == e{
                count -= 1;
            }
            sub_deq.push_back(e);
        }
        return Ok(sub_deq)
    }
    pub(crate) fn split_deque_by_ele(deq:&mut VecDeque<Element>,le:Element,re:Element)->anyhow::Result<VecDeque<Element>> {
        let mut sub_deq = VecDeque::new();
        let mut count = 1i32;
        while let Some(e) = deq.pop_front() {
            if e == le{
                count += 1;
            }else if e == re{
                count -= 1;
            }
            if count == 0 {
                return Ok(sub_deq)
            }
            sub_deq.push_back(e);
        }
        return anyhow!("element[{le:?} {re:?}] mismatch").err()
    }

    pub(crate) fn expression_parse(expr:String)->anyhow::Result<Calc>{
        let mut deq = Self::expression_split(expr)?;

        print!("expression_split -->");
        for i in deq.iter(){
            print!("_{}_",i.to_string());
        }
        println!(" <--- over");

        Self::convert_one_group_calc(None,&mut deq)
    }
}

#[cfg(test)]
mod test{
    use crate::{Calc};

    //cargo test --color=always --lib calc_parse::test::test_parse_remove_annotation --no-fail-fast --  --exact  unstable-options --nocapture
    #[test]
    fn test_parse_remove_annotation(){
        let data = "/*start*/he/*\n*/llo /**/wo/**/rld/* *//*end*/".into();
        let s = Calc::parse_remove_annotation(data).unwrap();
        assert_eq!(s,"hello world");
    }
    //cargo test --color=always --lib calc_parse::test::test_parse_remove_annotation_error --no-fail-fast --  --exact  unstable-options --nocapture
    #[test]
    #[should_panic]
    fn test_parse_remove_annotation_error(){
        let data = "/*/".into();
        Calc::parse_remove_annotation(data).unwrap();
    }

    //cargo test --color=always --lib calc_parse::test::test_parse_split --no-fail-fast --  --exact  unstable-options --nocapture
    #[test]
    fn test_parse_split(){
        let data = "/*注释1*/ a+b/*注释2*/> c ||\
         b &/*注释3*/& c > ab * ( strlen( a ,b , c) ) /*注释4*/ \n\
         || (\"asdd\" > 10 < a * 0.12) || s < [1,2,3]";
        let deq = Calc::expression_split(data.to_string()).unwrap();
        println!("test expression --> {data}");
        print!("split result -->");
        for i in deq.iter(){
            print!("_{}_",i.to_string());
        }
        println!(" <--- over")
    }

    //cargo test --color=always --lib calc_parse::test::test_expression_parse --no-fail-fast --  --exact  unstable-options --nocapture
    #[test]
    fn test_expression_parse(){
        let expr = "a > b || c <= d || eft > strlen(\"hello\",\"world\",2) || true || ~x >> y > -z || (n && m < n*m) || in(a,[1>2,3,\"123\"])";
        let calc = Calc::expression_parse(expr.into()).unwrap();
        println!("--->{}",calc.to_string());
        // println!(" '~' < '||' {}",Opt::REV > Opt::OT);
    }

    //cargo test --color=always --lib calc_parse::test::test_expression_simple --no-fail-fast --  --exact  unstable-options --nocapture
    #[test]
    fn test_expression_simple(){
        let expr = "age > 18 && native_place == \"中国\"";
        let calc = Calc::expression_parse(expr.into()).unwrap();
        println!("--->{}",calc.to_string());
    }
}
