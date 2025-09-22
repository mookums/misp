use criterion::{Criterion, criterion_group, criterion_main};
use misp_interp::Misp;

fn criterion_benchmark(c: &mut Criterion) {
    let mut misp = Misp::default();

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
        b.iter(|| {
            misp.eval("(+ (+ (+ 1 2) (+ 3 4)) (+ (+ 5 6) (+ 7 8)))")
                .unwrap()
        });
    });

    c.bench_function("interp sqrt operation", |b| {
        b.iter(|| misp.eval("(sqrt 10)").unwrap());
    });

    c.bench_function("interp power operation", |b| {
        b.iter(|| misp.eval("(pow 2 8)").unwrap());
    });

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
        misp.eval("(func square (x) (* x x))").unwrap();
        b.iter(|| misp.eval("(square 15)").unwrap());
    });

    c.bench_function("interp small summate", |b| {
        b.iter(|| misp.eval("(summate 0 10 sqrt)").unwrap());
    });

    c.bench_function("interp pi constant", |b| {
        b.iter(|| misp.eval("pi").unwrap());
    });

    c.bench_function("interp comparison", |b| {
        b.iter(|| misp.eval("(> 10 5)").unwrap());
    });

    c.bench_function("interp runtime factorial", |b| {
        misp.eval("(func factorialRuntime (n) (if (<= n 1) 1 (* n (factorialRuntime (- n 1)))))")
            .unwrap();
        b.iter(|| misp.eval("(factorialRuntime 1000)").unwrap());
    });

    c.bench_function("interp builtin factorial", |b| {
        b.iter(|| misp.eval("(factorial 1000)").unwrap());
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
