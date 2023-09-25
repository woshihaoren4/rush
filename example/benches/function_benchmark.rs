use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};
use expr_engine::ExprEngine;
use rush_core::{Function, FunctionSet, RuleFlow, Rush};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::sync::Arc;

const FUNCTION_RULE: &'static str = "
    rule FUNCTION_RULE
    when
        str_rev(country)=='加拿大';
        str_splice(str_rev(country),city)=='加拿大 多伦多';
        abs(revenue.low) < 100 && abs(revenue.high) > 1000;
    then
        message = str_splice(str_rev(country),city,'贫富差距大');
        revenue.avg = abs(revenue.low) + abs(revenue.high) / 2;
    ";
pub fn str_rev(s: String) -> anyhow::Result<String> {
    let mut result = String::new();
    let si = s.chars();
    for s in si.rev() {
        result.push(s);
    }
    Ok(result)
}
pub struct StrSplice(char);
impl Function for StrSplice {
    fn call(&self, _fs: Arc<dyn FunctionSet>, args: Vec<Value>) -> anyhow::Result<Value> {
        let mut res = String::new();
        for i in args {
            if !res.is_empty() {
                res.push(self.0);
            }
            if let Value::String(s) = i {
                res += s.as_str();
            }
        }
        Ok(Value::String(res))
    }
}

#[derive(Deserialize, Serialize)]
pub struct Resp {
    #[serde(default = "Default::default")]
    message: String,
    revenue: Revenue,
}
#[derive(Deserialize, Serialize)]
pub struct Revenue {
    avg: i64,
}
fn have_function_rush(rh: &Rush) {
    let resp: Resp = rh
        .flow(
            r#"{"country":"大拿加","city":"多伦多","revenue":{ "low":-10,"high":1002 } }"#
                .parse::<Value>()
                .unwrap(),
        )
        .unwrap();

    assert_eq!(resp.message.as_str(), "加拿大 多伦多 贫富差距大");
    assert_eq!(resp.revenue.avg, 506);
}

fn criterion_benchmark(c: &mut Criterion) {
    let ee = ExprEngine::from([FUNCTION_RULE]);
    let rh = Rush::from(ee);

    let rh = rh
        .raw_register_function("str_splice", StrSplice(' '))
        .register_function("str_rev", str_rev)
        .register_function("abs", |i: i64| Ok(i.abs()));

    c.bench_with_input(
        BenchmarkId::new("have_function_rush", "have_function_rush"),
        &rh,
        |b, s| {
            b.iter(|| have_function_rush(s));
        },
    );
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
