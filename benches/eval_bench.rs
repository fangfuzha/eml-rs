use criterion::{black_box, criterion_group, criterion_main, Criterion};
use eml_rs::bytecode::BytecodeProgram;
use eml_rs::ir::{eval_rpn_complex, Expr};
use eml_rs::lowering::{cross_entropy_template, lower_to_eml, SourceExpr};
use num_complex::Complex64;

fn positive_real_samples(n: usize, arity: usize) -> Vec<Vec<Complex64>> {
    (0..n)
        .map(|i| {
            (0..arity)
                .map(|j| {
                    let x = 0.1 + (i as f64) * 0.001 + (j as f64) * 0.05;
                    Complex64::new(x, 0.0)
                })
                .collect()
        })
        .collect()
}

fn bench_eml_ln_tree(c: &mut Criterion) {
    let expr = Expr::ln(Expr::var(0));
    let samples = positive_real_samples(1_024, 1);

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
    let samples = positive_real_samples(1_024, 1);

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
    let samples = positive_real_samples(1_024, 1);

    c.bench_function("eml_ln_bytecode_eval", |b| {
        b.iter(|| {
            for vars in &samples {
                black_box(prog.eval_complex(black_box(vars)).unwrap());
            }
        })
    });
}

fn bench_native_ln(c: &mut Criterion) {
    let samples = positive_real_samples(1_024, 1);

    c.bench_function("native_complex_ln", |b| {
        b.iter(|| {
            for vars in &samples {
                black_box(vars[0].ln());
            }
        })
    });
}

fn build_softmax_ce_expr() -> Expr {
    let logits = vec![
        SourceExpr::var(0),
        SourceExpr::var(1),
        SourceExpr::var(2),
        SourceExpr::var(3),
    ];
    let ce = cross_entropy_template(&logits, 2).unwrap();
    lower_to_eml(&ce).unwrap()
}

fn bench_softmax_ce_tree(c: &mut Criterion) {
    let expr = build_softmax_ce_expr();
    let samples = positive_real_samples(1_024, 4);

    c.bench_function("softmax_ce_tree_eval", |b| {
        b.iter(|| {
            for vars in &samples {
                black_box(expr.eval_complex(black_box(vars)).unwrap());
            }
        })
    });
}

fn bench_softmax_ce_bytecode(c: &mut Criterion) {
    let expr = build_softmax_ce_expr();
    let prog = BytecodeProgram::from_expr(&expr).unwrap();
    let samples = positive_real_samples(1_024, 4);

    c.bench_function("softmax_ce_bytecode_eval", |b| {
        b.iter(|| {
            for vars in &samples {
                black_box(prog.eval_complex(black_box(vars)).unwrap());
            }
        })
    });
}

criterion_group!(
    benches,
    bench_eml_ln_tree,
    bench_eml_ln_rpn,
    bench_eml_ln_bytecode,
    bench_native_ln,
    bench_softmax_ce_tree,
    bench_softmax_ce_bytecode
);
criterion_main!(benches);
