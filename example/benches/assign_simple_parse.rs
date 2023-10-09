use criterion::{criterion_group, criterion_main, Criterion};
use rush_expr_engine::Assign;

fn assign_simple_parse() {
    let _a = "msg='success'"
        .parse::<Assign>()
        .expect("assign_simple_parse panic:");
}

fn assign_full_parse() {
    let exec_expression = r#"
        data.message = 'success';
        data.code = 0;
        data.value1 = [1,2,3];
        data.value2 = args1 + args2;
        data.value3 = !args3;
        data.value4 = str_len('hello world');
        data.value5 = 1>>2;
        "#;
    let _a = exec_expression
        .parse::<Assign>()
        .expect("new Assign failed");
}

fn criterion_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("assign_parse_benchmark");
    group.significance_level(0.1).sample_size(100);

    group.bench_function("assign_simple_parse", |b| b.iter(|| assign_simple_parse()));
    group.bench_function("rule_full_parse", |b| b.iter(|| assign_full_parse()));
    group.finish();
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
