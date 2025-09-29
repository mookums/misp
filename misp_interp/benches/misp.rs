use criterion::{Criterion, criterion_group, criterion_main};
use misp_interp::Misp;

fn criterion_benchmark(c: &mut Criterion) {
    let mut misp = Misp::default();
    misp.eval("(load math)").unwrap();

    c.bench_function("interp basic number", |b| {
        b.iter(|| misp.eval("0").unwrap());
    });

    c.bench_function("interp basic add", |b| {
        b.iter(|| misp.eval("(+ 10 10)").unwrap());
    });

    c.bench_function("interp basic subtract", |b| {
        b.iter(|| misp.eval("(- 100 25)").unwrap());
    });

    c.bench_function("interp basic multiply", |b| {
        b.iter(|| misp.eval("(* 7 8)").unwrap());
    });

    c.bench_function("interp basic divide", |b| {
        b.iter(|| misp.eval("(/ 100 4)").unwrap());
    });

    c.bench_function("interp nested arithmetic", |b| {
        b.iter(|| misp.eval("(+ (* 5 6) (- 20 8))").unwrap());
    });

    c.bench_function("interp deep nesting", |b| {
        b.iter(|| misp.eval("(+ 1 2 3 4 5 6 7 8)").unwrap());
    });

    // c.bench_function("interp sqrt operation", |b| {
    //     b.iter(|| misp.eval("(sqrt 10)").unwrap());
    // });

    // c.bench_function("interp perfect sqrt operation", |b| {
    //     b.iter(|| misp.eval("(sqrt 157772167)").unwrap());
    // });

    // c.bench_function("interp power operation", |b| {
    //     b.iter(|| misp.eval("(pow 2 8)").unwrap());
    // });

    c.bench_function("interp decimal addition", |b| {
        b.iter(|| misp.eval("(+ 0.1 0.2)").unwrap());
    });

    c.bench_function("interp high precision multiply", |b| {
        b.iter(|| {
            misp.eval("(* 3.141592653589793 2.718281828459045)")
                .unwrap()
        });
    });

    c.bench_function("interp function call", |b| {
        misp.eval("(func quadruple (x) (* 4 x))").unwrap();
        b.iter(|| misp.eval("(quadruple 15)").unwrap());
    });

    c.bench_function("interp small summate", |b| {
        b.iter(|| misp.eval("(summate 0 10 square)").unwrap());
    });

    c.bench_function("interp pi constant", |b| {
        b.iter(|| misp.eval("math::pi").unwrap());
    });

    c.bench_function("interp comparison", |b| {
        b.iter(|| misp.eval("(> 10 5)").unwrap());
    });

    c.bench_function("interp runtime fibonacci", |b| {
        misp.eval("(func fibRt (n) (if (<= n 1) n (+ (fib (- n 2)) (fibRt (- n 1)))))")
            .unwrap();
        b.iter(|| misp.eval("(fibRt 10)").unwrap());
    });

    c.bench_function("interp fib stdlib", |b| {
        b.iter(|| misp.eval("(fib 10)").unwrap());
    });

    c.bench_function("interp runtime factorial", |b| {
        misp.eval("(func factorialRt (n) (if (<= n 1) 1 (* n (factorialRt (- n 1)))))")
            .unwrap();
        b.iter(|| misp.eval("(factorialRt 1000)").unwrap());
    });

    c.bench_function("interp factorial stdlib", |b| {
        b.iter(|| misp.eval("(factorial 1000)").unwrap());
    });

    // c.bench_function("interp builtin factorial", |b| {
    //     b.iter(|| misp.eval("(factorial 1000)").unwrap());
    // });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
