use std::collections::VecDeque;
use crate::{Calc, Element};

pub trait CalcBuilderEvent{
    fn remove_annotation_before(&self,_expr:&mut String)->anyhow::Result<()>{Ok(())}
    //disable_fast_parse_annotation == ture 才会触发这个动作
    fn remove_annotation_after(&self,_expr:&mut String)->anyhow::Result<()>{Ok(())}
    fn expression_split_check(&self,_deq:&mut VecDeque<Element>)->anyhow::Result<()>{Ok(())}
    fn calc_check(&self,_calc:&mut Calc)->anyhow::Result<()>{Ok(())}
}
impl CalcBuilderEvent for (){}


/// CalcBuilder 构建过程
/// 1. 去掉注释
/// 2. 表达式拆分到队列
/// 3. 队列重组为运算树
#[derive(Debug,Default)]
pub struct CalcBuilder{
    //true: 关闭快速注解解析模式
    // 在快速解析中 例如："a > b &/*注释*/& d < c" 这样的注解会解析失败
    disable_fast_parse_annotation:bool,

    //需要解析的表达式
    expr : String,
}

impl CalcBuilder {
    pub fn new<S:Into<String>>(expr:S)->Self{
        Self{
            expr:expr.into(),
            ..Default::default()
        }
    }

    pub fn disable_fast_parse_annotation(mut self)->Self{
        self.disable_fast_parse_annotation = true;self
    }

}

impl CalcBuilder {
    pub fn build(self)->anyhow::Result<Calc>{
        self.build_event::<()>(None)
    }
    pub fn build_event<E:CalcBuilderEvent>(self,event:Option<E>)->anyhow::Result<Calc>{
        let  Self{
            disable_fast_parse_annotation,
            mut expr
        } = self;

        if let Some(ref e) = event{ //<<-------- 开始前检查
            e.remove_annotation_before(&mut expr)?;
        }

        if disable_fast_parse_annotation {
            expr = Calc::parse_remove_annotation(expr)?;

            if let Some(ref e) = event{ //<<---------- 去注释后检查
                e.remove_annotation_after(&mut expr)?;
            }
        }

        let mut deq = Calc::expression_split(expr)?;

        if let Some(ref e) = event{ //<<---------- 拆解检查
            e.expression_split_check(&mut deq)?;
        }

        let mut calc = Calc::convert_one_group_calc(None,&mut deq)?;

        if let Some(ref e) = event{ //<<---------- 算子检查
            e.calc_check(&mut calc)?;
        }

        Ok(calc)
    }
}