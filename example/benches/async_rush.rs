use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};
use expr_engine::ExprEngine;
use rush_core::{Filter, MultiRush, Rush};
use serde::Deserialize;
use serde_json::Value;

pub const MANY_RULE_ONE: &str = "
    rule MANY_RULE_ONE
    when
        country == '美国';
        age <= 18;
    then
        tag = '美国的年轻人'
    ";

pub const MANY_RULE_TWO: &str = "
    rule MANY_RULE_TWO
    when
        country == '美国';
        age > 18 && age < 30;
    then
        tag = '美国的青年人'
    ";

pub const MANY_RULE_THREE: &str = "
    rule MANY_RULE_THREE
    when
        country == '中国';
        age <= 18;
    then
        tag = '中国的年轻人'
    ";
pub const MANY_RULE_FOUR: &str = "
    rule MANY_RULE_FOUR
    when
        country == '中国';
        age > 18 && age < 30;
    then
        tag = '中国的青年人'
    ";
#[derive(Deserialize)]
struct Tag {
    #[serde(default = "Default::default")]
    tag: String,
}
async fn multi_flow(rh: &MultiRush) {
    let res: Tag = rh
        .multi_flow(r#"{"country":"美国","age":17}"#.parse::<Value>().unwrap())
        .await
        .unwrap();
    assert_eq!(
        res.tag.as_str(),
        "美国的年轻人",
        r#"case : {{"country":"美国","age":17}} failed"#
    );
}

fn sync_flow(rh: &Rush) {
    let res: Tag = rh
        .flow(r#"{"country":"美国","age":17}"#.parse::<Value>().unwrap())
        .unwrap();
    assert_eq!(
        res.tag.as_str(),
        "美国的年轻人",
        r#"case : {{"country":"美国","age":17}} failed"#
    );
}

fn criterion_benchmark(c: &mut Criterion) {
    let rh = Rush::from(Into::<ExprEngine>::into([
        MANY_RULE_ONE,
        MANY_RULE_TWO,
        MANY_RULE_THREE,
        MANY_RULE_FOUR,
    ]));
    let mrh: MultiRush = rh.into();

    c.bench_with_input(
        BenchmarkId::new("multi_flow", "multi_flow"),
        &mrh,
        |b, s| {
            b.to_async(tokio::runtime::Runtime::new().unwrap())
                .iter(|| multi_flow(s));
        },
    );

    let rh = Rush::from(Into::<ExprEngine>::into([
        MANY_RULE_ONE,
        MANY_RULE_TWO,
        MANY_RULE_THREE,
        MANY_RULE_FOUR,
    ]));

    c.bench_with_input(BenchmarkId::new("sync_flow", "sync_flow"), &rh, |b, s| {
        b.iter(|| sync_flow(s));
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
