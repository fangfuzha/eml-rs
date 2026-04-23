//! Minimal symbolic-regression style training loop with a parameterized EML tree.
//!
//! Model template:
//! `f(x) = eml(w0*x + w1, exp(w2*x + w3))`
//!        = exp(w0*x + w1) - (w2*x + w3)
//!
//! This example demonstrates how EML-based structures can be optimized with
//! simple finite-difference gradients before introducing full autodiff/search.

use eml_rs::core::{eml_real_with_policy, EvalPolicy};

#[derive(Debug, Clone, Copy)]
struct Params {
    w0: f64,
    w1: f64,
    w2: f64,
    w3: f64,
}

fn predict(p: Params, x: f64) -> f64 {
    let lhs = p.w0 * x + p.w1;
    let rhs = (p.w2 * x + p.w3).exp();
    let relaxed = EvalPolicy::relaxed();
    match eml_real_with_policy(lhs, rhs, &relaxed) {
        Ok(v) if v.is_finite() => v.clamp(-1.0e6, 1.0e6),
        _ => {
            if lhs.is_sign_positive() {
                1.0e6
            } else {
                -1.0e6
            }
        }
    }
}

fn mse_loss(p: Params, dataset: &[(f64, f64)]) -> f64 {
    let mut acc = 0.0;
    for (x, y) in dataset {
        let e = predict(p, *x) - *y;
        acc += e * e;
    }
    acc / dataset.len() as f64
}

fn finite_diff_grad(p: Params, dataset: &[(f64, f64)], eps: f64) -> Params {
    let mut grad = Params {
        w0: 0.0,
        w1: 0.0,
        w2: 0.0,
        w3: 0.0,
    };
    let base = mse_loss(p, dataset);

    let p_w0 = Params {
        w0: p.w0 + eps,
        ..p
    };
    let p_w1 = Params {
        w1: p.w1 + eps,
        ..p
    };
    let p_w2 = Params {
        w2: p.w2 + eps,
        ..p
    };
    let p_w3 = Params {
        w3: p.w3 + eps,
        ..p
    };

    grad.w0 = (mse_loss(p_w0, dataset) - base) / eps;
    grad.w1 = (mse_loss(p_w1, dataset) - base) / eps;
    grad.w2 = (mse_loss(p_w2, dataset) - base) / eps;
    grad.w3 = (mse_loss(p_w3, dataset) - base) / eps;
    grad
}

fn main() {
    let target = Params {
        w0: 1.3,
        w1: -0.2,
        w2: -0.7,
        w3: 0.4,
    };

    let dataset: Vec<(f64, f64)> = (-20..=20)
        .map(|k| {
            let x = k as f64 / 10.0;
            (x, predict(target, x))
        })
        .collect();

    let mut p = Params {
        w0: 0.4,
        w1: 0.5,
        w2: 0.2,
        w3: -0.1,
    };
    let lr = 3e-4;
    let eps = 1e-5;

    for epoch in 0..400 {
        let grad = finite_diff_grad(p, &dataset, eps);
        let clip = |g: f64| g.clamp(-50.0, 50.0);
        p.w0 -= lr * clip(grad.w0);
        p.w1 -= lr * clip(grad.w1);
        p.w2 -= lr * clip(grad.w2);
        p.w3 -= lr * clip(grad.w3);

        if epoch % 50 == 0 || epoch == 399 {
            let loss = mse_loss(p, &dataset);
            println!("epoch={epoch:03} loss={loss:.8e} params={p:?}");
        }
    }

    println!("target={target:?}");
    println!("learned={p:?}");
}
