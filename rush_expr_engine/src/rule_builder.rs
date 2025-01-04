use crate::{Assign, Calc};
use anyhow::anyhow;
use wd_tools::PFErr;

const RULE_FORMAT: &str = "\n\
The keyword cannot be repeated: when,then
rule [name] [description] [engine/default:expr] [...]
when
    [condition 1];
    [condition 2];
    ...
    [condition n];
then
    [key1 = execute 1];
    [key2 = execute 2];
    ...
    [keyn = execute n];
";
const EXPR_ENGINE: &str = "expr";
const RULE_TAG: &str = "rule";

#[derive(Debug, Default)]
pub struct ExprEngine {
    rules: Vec<(String, Vec<Calc>, Assign)>,
}

impl ExprEngine {
    pub fn insert_rule<S: Into<String>>(&mut self, name: S, calc: Vec<Calc>, assign: Assign) {
        self.rules.push((name.into(), calc, assign));
    }
    pub fn register_rule_by_expr<S: Into<String>, E: AsRef<str>>(
        &mut self,
        name: S,
        calc: E,
        exec: E,
    ) -> anyhow::Result<()> {
        //先解析calc
        let ss: Vec<_> = calc.as_ref().split(";").collect();
        let mut calc = vec![];
        for s in ss {
            let s = s.trim_matches(|c| " \n\t\r".contains(c));
            if s.is_empty() {
                continue;
            }
            calc.push(s.parse()?);
        }
        let assign = exec.as_ref().parse()?;
        self.insert_rule(name.into(), calc, assign);
        Ok(())
    }
    pub fn register_rule<R: AsRef<str>>(&mut self, rule: R) -> anyhow::Result<()> {
        let rule = rule.as_ref();
        // 先解头
        let ce: Vec<_> = rule.split("when").collect();
        if ce.len() != 2 {
            return anyhow!(
                "rule[{}] format error,format that can be parsed：{}",
                rule,
                RULE_FORMAT
            )
            .err();
        }
        let hs: Vec<_> = ce[0]
            .trim_matches(|c| " \n\r\t".contains(c))
            .split(" ")
            .collect();
        if hs.len() <= 1 {
            return anyhow!("not found rule name").err();
        }
        if hs[0].trim_matches(|c| " \n\r\t".contains(c)).to_lowercase() != RULE_TAG {
            return anyhow!("rule must start with 'rule'").err();
        }
        if hs.len() >= 4 {
            if hs[3].to_lowercase() != EXPR_ENGINE {
                return anyhow!("ExprEngine only support expression parse").err();
            }
        }
        let name = hs[1].to_string();
        //再解析条件
        let ce: Vec<_> = ce[1].split("then").collect();
        if ce.len() != 2 {
            return anyhow!(
                "rule[{}] format error,format that can be parsed：{}",
                rule,
                RULE_FORMAT
            )
            .err();
        }
        // let cs:Vec<_> = ce[0].split(";").collect();
        self.register_rule_by_expr(name, ce[0], ce[1])?;
        Ok(())
    }
}

impl IntoIterator for ExprEngine {
    type Item = (String, Vec<Calc>, Assign);
    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.rules
            .into_iter()
            .map(|(n, c, a)| (n, c, a))
            .collect::<Vec<_>>()
            .into_iter()
    }
}

impl<'a, S: IntoIterator<Item = &'a str>> From<S> for ExprEngine {
    fn from(value: S) -> Self {
        let mut ee = ExprEngine::default();
        for i in value {
            ee.register_rule(i).expect("from item parse rule failed");
        }
        ee
    }
}

#[cfg(test)]
mod test {
    use crate::ExprEngine;

    #[test]
    fn test_expr_engine_from() {
        let rule1 = "rule rule1 第一条规则\
        when
            a > b || d < c;
            tag == '便签1';
        then
            data.code = 0
        ";

        let rule2 = "rule rule2 第二条规则\
        when
            tag == '便签2'
        then
            data.message = 'success';
            data.total = len(request.list);
        ";

        let ee = ExprEngine::from([rule1, rule2]);
        for ((name, cs, a)) in ee.rules {
            println!("---> rule {}", name);
            println!("when");
            for i in cs {
                println!("    {}", i.to_string());
            }
            println!("then");
            println!("    {:?}", a);
            println!("<--- over");
        }
    }
}
