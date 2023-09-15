use criterion::{black_box, criterion_group, criterion_main, Criterion};
use calc_rule::Calc;


fn single_parse(){
    let _calc:Calc = "1".parse().unwrap();
}

fn simple_parse(){
    let _calc:Calc = "(requests_made * requests_succeeded / 100) >= 90".parse().unwrap();
}

fn full_parse(){
    let _calc:Calc = r#"(args1 != "hello world"
                  || utc("2023-01-02") > utc(args2))
                  && (in(args3,[1,2,3,4,"helle","world"])
                  || 3.14 > args4 >> 2 || !args5 || false )"#
        .parse().unwrap();
}

fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("single_parse", |b| {
        b.iter(||single_parse())
    });
    c.bench_function("simple_parse",|b|{
        b.iter(||simple_parse())
    });
    c.bench_function("full_parse",|b|{
        b.iter(||full_parse())
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);