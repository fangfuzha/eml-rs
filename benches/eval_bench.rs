use criterion::{black_box, criterion_group, criterion_main, Criterion};
use eml_rs::bytecode::BytecodeProgram;
use eml_rs::ir::{eval_rpn_complex, Expr};
use num_complex::Complex64;

fn positive_real_samples(n: usize) -> Vec<Vec<Complex64>> {
    (0..n)
        .map(|i| {
            let x = 0.1 + (i as f64) * 0.001;
            vec![Complex64::new(x, 0.0)]
        })
        .collect()
}

fn bench_eml_ln_tree(c: &mut Criterion) {
    let expr = Expr::ln(Expr::var(0));
    let samples = positive_real_samples(1_024);

    c.bench_function("eml_ln_tree_eval", |b| {
        b.iter(|| {
            for vars in &samples {
                black_box(expr.eval_complex(black_box(vars)).unwrap());
            }
        })
    });
}

fn bench_eml_ln_rpn(c: &mut Criterion) {
    let expr = Expr::ln(Expr::var(0));
    let tokens = expr.to_rpn_vec();
    let samples = positive_real_samples(1_024);

    c.bench_function("eml_ln_rpn_eval", |b| {
        b.iter(|| {
            for vars in &samples {
                black_box(eval_rpn_complex(black_box(&tokens), black_box(vars)).unwrap());
            }
        })
    });
}

fn bench_eml_ln_bytecode(c: &mut Criterion) {
    let expr = Expr::ln(Expr::var(0));
    let prog = BytecodeProgram::from_expr(&expr).unwrap();
    let samples = positive_real_samples(1_024);

    c.bench_function("eml_ln_bytecode_eval", |b| {
        b.iter(|| {
            for vars in &samples {
                black_box(prog.eval_complex(black_box(vars)).unwrap());
            }
        })
    });
}

fn bench_native_ln(c: &mut Criterion) {
    let samples = positive_real_samples(1_024);

    c.bench_function("native_complex_ln", |b| {
        b.iter(|| {
            for vars in &samples {
                black_box(vars[0].ln());
            }
        })
    });
}

criterion_group!(
    benches,
    bench_eml_ln_tree,
    bench_eml_ln_rpn,
    bench_eml_ln_bytecode,
    bench_native_ln
);
criterion_main!(benches);
