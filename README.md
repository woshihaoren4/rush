# Rush
Rush is a universal rules engine.

Rush provides a general computational abstraction. There can be multiple implementations and any combination.

Conditional parallel computing

## Quick start

```rust
const SIMPLE_RULE: &'static str = "
rule COMPLEX_RULE
when
    age > 18
then
    stage = 'adult'
";
fn main(){
    
    let rh = Rush::from(Into::<ExprEngine>::into([SIMPLE_RULE]));

    let res:HashMap<String,String> = rh.flow(r#"{"age":19}"#.parse::<Value>().unwrap()).unwrap();
    
    assert_eq!(res.get("stage").unwrap().as_str(),"adult");

}
```

Here, regular expressions are generated directly. You can actually implement some of them yourself to integrate with your own services.

[More examples](https://github.com/woshihaoren4/rush/tree/main/example/src)

## Keywords

The direct parsing rules are as follows:
```rust
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
```

## Operators
- Modifiers: + - / * & | ^ % >> << 
- Comparators: > >= < <= == !=
- Logical ops: || &&
- Numeric constants, as i64, if have '.' as f64
- String constants (single quotes: 'foobar')
- Boolean constants: true false
- Parenthesis to control order of evaluation ( )
- Arrays [anything separated by , within parenthesis: [1, 2, 'foo']]
- Prefixes: ! - ~
- Null coalescence: null
- Function: function_name(args)result

## Function

You can add functions just like normal rust functions

```rust
    let rh = rh
        .register_function("abs", |i: i64| Ok(i.abs()));
```

[More function impl](https://github.com/woshihaoren4/rush/blob/main/example/src/function_test.rs)

## Abstraction and Structure

![img.png](img.png)

## Benchmark

If you're concerned about the overhead of this library, a good range of benchmarks are built into this repo. You can run them with `cargo bench -- --verbose` The library is built with an eye towards being quick, but has not been aggressively profiled and optimized. For most applications, though, it is completely fine.

[benchmark detail](https://github.com/woshihaoren4/rush/tree/main/example/benches)

Here are my test results，at MacBook Pro,CPU 2.6 GHz Intel Core i7, [lowest, average, highest]

```bash
assign_simple_parse  time:   [620.70 ns 625.08 ns 630.18 ns]
rule_full_parse      time:   [7.5513 µs 7.5794 µs 7.6094 µs]
multi_flow           time:   [15.363 µs 15.721 µs 16.184 µs]
sync_flow            time:   [2.9953 µs 3.0295 µs 3.0700 µs]
single_parse         time:   [165.08 ns 174.83 ns 186.49 ns]
simple_parse         time:   [2.6358 µs 2.6470 µs 2.6591 µs]
full_parse           time:   [19.868 µs 20.089 µs 20.356 µs]
have_function_rush   time:   [6.9074 µs 6.9507 µs 7.0011 µs]
```

## Plan

Expressions are currently supported as cells of count. Future support for lua and wasm is also planned。

## License
This project is licensed under the MIT general use license. You're free to integrate, fork, and play with this code as you feel fit without consulting the author, as long as you provide proper credit to the author in your works.