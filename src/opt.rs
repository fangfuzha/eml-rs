//! Source-level rewrite rules and lightweight cost model.
//!
//! The rewrite objective is to reduce expensive `exp/log` usage before lowering
//! into pure EML trees.

use crate::lowering::{eval_source_expr_complex, SourceExpr};
use num_complex::Complex64;

/// Heuristic cost metrics used for rewrite decisions.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct CostModel {
    /// Estimated number of `exp` invocations after lowering.
    pub exp_calls: usize,
    /// Estimated number of `log` invocations after lowering.
    pub log_calls: usize,
    /// Weighted scalar score (`exp`/`log` prioritized).
    pub score: usize,
}

/// Estimates lowering cost for a source expression.
pub fn estimate_cost(expr: &SourceExpr) -> CostModel {
    fn walk(expr: &SourceExpr) -> (usize, usize) {
        match expr {
            SourceExpr::Var(_)
            | SourceExpr::Int(_)
            | SourceExpr::Rational(_, _)
            | SourceExpr::ConstE
            | SourceExpr::ConstI
            | SourceExpr::ConstPi => (0, 0),
            SourceExpr::Neg(x) => {
                let (e, l) = walk(x);
                (e + 2, l + 2)
            }
            SourceExpr::Add(a, b) => {
                let (ea, la) = walk(a);
                let (eb, lb) = walk(b);
                (ea + eb + 4, la + lb + 4)
            }
            SourceExpr::Sub(a, b) => {
                let (ea, la) = walk(a);
                let (eb, lb) = walk(b);
                (ea + eb + 2, la + lb + 2)
            }
            SourceExpr::Mul(a, b) => {
                let (ea, la) = walk(a);
                let (eb, lb) = walk(b);
                (ea + eb + 5, la + lb + 5)
            }
            SourceExpr::Div(a, b) => {
                let (ea, la) = walk(a);
                let (eb, lb) = walk(b);
                (ea + eb + 7, la + lb + 7)
            }
            SourceExpr::Pow(a, b) => {
                let (ea, la) = walk(a);
                let (eb, lb) = walk(b);
                (ea + eb + 6, la + lb + 6)
            }
            SourceExpr::Exp(x) => {
                let (e, l) = walk(x);
                (e + 1, l)
            }
            SourceExpr::Log(x) => {
                let (e, l) = walk(x);
                (e, l + 1)
            }
            SourceExpr::Sin(x) | SourceExpr::Cos(x) => {
                let (e, l) = walk(x);
                (e + 10, l + 10)
            }
            SourceExpr::Tan(x) => {
                let (e, l) = walk(x);
                (e + 24, l + 24)
            }
            SourceExpr::Sinh(x) | SourceExpr::Cosh(x) => {
                let (e, l) = walk(x);
                (e + 9, l + 9)
            }
            SourceExpr::Tanh(x) => {
                let (e, l) = walk(x);
                (e + 18, l + 18)
            }
            SourceExpr::Asin(x) | SourceExpr::Acos(x) | SourceExpr::Atan(x) => {
                let (e, l) = walk(x);
                (e + 30, l + 30)
            }
            SourceExpr::Sqrt(x) => {
                let (e, l) = walk(x);
                (e + 6, l + 6)
            }
            SourceExpr::Sigmoid(x) => {
                let (e, l) = walk(x);
                (e + 8, l + 8)
            }
            SourceExpr::Softplus(x) => {
                let (e, l) = walk(x);
                (e + 8, l + 8)
            }
            SourceExpr::Swish(x) => {
                let (e, l) = walk(x);
                (e + 16, l + 16)
            }
            SourceExpr::GeluTanh(x) => {
                let (e, l) = walk(x);
                (e + 36, l + 36)
            }
            SourceExpr::ReluSoft(x) => {
                let (e, l) = walk(x);
                (e + 8, l + 8)
            }
            SourceExpr::Elu(x, alpha) => {
                let (ex, lx) = walk(x);
                let (ea, la) = walk(alpha);
                (ex + ea + 20, lx + la + 20)
            }
            SourceExpr::LeakyRelu(x, slope) => {
                let (ex, lx) = walk(x);
                let (es, ls) = walk(slope);
                (ex + es + 14, lx + ls + 14)
            }
            SourceExpr::Softsign(x) => {
                let (e, l) = walk(x);
                (e + 14, l + 14)
            }
            SourceExpr::Mish(x) => {
                let (e, l) = walk(x);
                (e + 20, l + 20)
            }
        }
    }

    let (exp_calls, log_calls) = walk(expr);
    CostModel {
        exp_calls,
        log_calls,
        score: 4 * exp_calls + 4 * log_calls + exp_calls + log_calls,
    }
}

/// Applies one local rewrite pass to an expression.
///
/// The pass is cost-aware and prefers lower estimated cost when multiple
/// equivalent local forms are available.
pub fn rewrite_once(expr: &SourceExpr) -> SourceExpr {
    let rewritten_children = match expr {
        SourceExpr::Neg(x) => SourceExpr::Neg(Box::new(rewrite_once(x))),
        SourceExpr::Add(a, b) => {
            SourceExpr::Add(Box::new(rewrite_once(a)), Box::new(rewrite_once(b)))
        }
        SourceExpr::Sub(a, b) => {
            SourceExpr::Sub(Box::new(rewrite_once(a)), Box::new(rewrite_once(b)))
        }
        SourceExpr::Mul(a, b) => {
            SourceExpr::Mul(Box::new(rewrite_once(a)), Box::new(rewrite_once(b)))
        }
        SourceExpr::Div(a, b) => {
            SourceExpr::Div(Box::new(rewrite_once(a)), Box::new(rewrite_once(b)))
        }
        SourceExpr::Pow(a, b) => {
            SourceExpr::Pow(Box::new(rewrite_once(a)), Box::new(rewrite_once(b)))
        }
        SourceExpr::Exp(x) => SourceExpr::Exp(Box::new(rewrite_once(x))),
        SourceExpr::Log(x) => SourceExpr::Log(Box::new(rewrite_once(x))),
        SourceExpr::Sin(x) => SourceExpr::Sin(Box::new(rewrite_once(x))),
        SourceExpr::Cos(x) => SourceExpr::Cos(Box::new(rewrite_once(x))),
        SourceExpr::Tan(x) => SourceExpr::Tan(Box::new(rewrite_once(x))),
        SourceExpr::Sinh(x) => SourceExpr::Sinh(Box::new(rewrite_once(x))),
        SourceExpr::Cosh(x) => SourceExpr::Cosh(Box::new(rewrite_once(x))),
        SourceExpr::Tanh(x) => SourceExpr::Tanh(Box::new(rewrite_once(x))),
        SourceExpr::Asin(x) => SourceExpr::Asin(Box::new(rewrite_once(x))),
        SourceExpr::Acos(x) => SourceExpr::Acos(Box::new(rewrite_once(x))),
        SourceExpr::Atan(x) => SourceExpr::Atan(Box::new(rewrite_once(x))),
        SourceExpr::Sqrt(x) => SourceExpr::Sqrt(Box::new(rewrite_once(x))),
        SourceExpr::Sigmoid(x) => SourceExpr::Sigmoid(Box::new(rewrite_once(x))),
        SourceExpr::Softplus(x) => SourceExpr::Softplus(Box::new(rewrite_once(x))),
        SourceExpr::Swish(x) => SourceExpr::Swish(Box::new(rewrite_once(x))),
        SourceExpr::GeluTanh(x) => SourceExpr::GeluTanh(Box::new(rewrite_once(x))),
        SourceExpr::ReluSoft(x) => SourceExpr::ReluSoft(Box::new(rewrite_once(x))),
        SourceExpr::Elu(x, alpha) => {
            SourceExpr::Elu(Box::new(rewrite_once(x)), Box::new(rewrite_once(alpha)))
        }
        SourceExpr::LeakyRelu(x, slope) => {
            SourceExpr::LeakyRelu(Box::new(rewrite_once(x)), Box::new(rewrite_once(slope)))
        }
        SourceExpr::Softsign(x) => SourceExpr::Softsign(Box::new(rewrite_once(x))),
        SourceExpr::Mish(x) => SourceExpr::Mish(Box::new(rewrite_once(x))),
        leaf => leaf.clone(),
    };

    let candidate = apply_local_rules(&rewritten_children);
    let base_cost = estimate_cost(&rewritten_children);
    let cand_cost = estimate_cost(&candidate);
    if cand_cost.score <= base_cost.score {
        candidate
    } else {
        rewritten_children
    }
}

/// Rewrites to a fixed point (bounded iterations).
pub fn optimize_for_lowering(expr: &SourceExpr) -> SourceExpr {
    let mut cur = expr.clone();
    for _ in 0..24 {
        let next = rewrite_once(&cur);
        if next == cur {
            return cur;
        }
        cur = next;
    }
    cur
}

/// Sanity-check helper used by tests/examples: evaluate rewrite equivalence.
pub fn semantically_equivalent_on_sample(
    a: &SourceExpr,
    b: &SourceExpr,
    vars: &[Complex64],
    tol: f64,
) -> bool {
    match (
        eval_source_expr_complex(a, vars),
        eval_source_expr_complex(b, vars),
    ) {
        (Ok(av), Ok(bv)) => (av - bv).norm() <= tol,
        _ => false,
    }
}

fn apply_local_rules(expr: &SourceExpr) -> SourceExpr {
    match expr {
        SourceExpr::Neg(x) => {
            if let SourceExpr::Neg(inner) = x.as_ref() {
                return *inner.clone();
            }
            if let Some(v) = as_rational(x) {
                return from_rational(-v.0, v.1);
            }
            expr.clone()
        }
        SourceExpr::Add(a, b) => {
            if is_zero(a) {
                return *b.clone();
            }
            if is_zero(b) {
                return *a.clone();
            }
            if let (Some(ra), Some(rb)) = (as_rational(a), as_rational(b)) {
                return from_rational(ra.0 * rb.1 + rb.0 * ra.1, ra.1 * rb.1);
            }
            expr.clone()
        }
        SourceExpr::Sub(a, b) => {
            if is_zero(b) {
                return *a.clone();
            }
            if let (Some(ra), Some(rb)) = (as_rational(a), as_rational(b)) {
                return from_rational(ra.0 * rb.1 - rb.0 * ra.1, ra.1 * rb.1);
            }
            expr.clone()
        }
        SourceExpr::Mul(a, b) => {
            if is_zero(a) || is_zero(b) {
                return SourceExpr::Int(0);
            }
            if is_one(a) {
                return *b.clone();
            }
            if is_one(b) {
                return *a.clone();
            }
            if let (Some(ra), Some(rb)) = (as_rational(a), as_rational(b)) {
                return from_rational(ra.0 * rb.0, ra.1 * rb.1);
            }
            expr.clone()
        }
        SourceExpr::Div(a, b) => {
            if is_zero(a) {
                return SourceExpr::Int(0);
            }
            if is_one(b) {
                return *a.clone();
            }
            if let (Some(ra), Some(rb)) = (as_rational(a), as_rational(b)) {
                if rb.0 != 0 {
                    return from_rational(ra.0 * rb.1, ra.1 * rb.0);
                }
            }
            expr.clone()
        }
        SourceExpr::Pow(a, b) => {
            if is_zero(b) {
                return SourceExpr::Int(1);
            }
            if is_one(b) {
                return *a.clone();
            }
            if let (Some((p, q)), Some((n, d))) = (as_rational(a), as_rational(b)) {
                if d == 1 && (-8..=8).contains(&n) {
                    if n >= 0 {
                        return from_rational(pow_i64(p, n as u32), pow_i64(q, n as u32));
                    }
                    let m = (-n) as u32;
                    return from_rational(pow_i64(q, m), pow_i64(p, m));
                }
            }
            expr.clone()
        }
        SourceExpr::Exp(x) => {
            if let SourceExpr::Log(inner) = x.as_ref() {
                return *inner.clone();
            }
            expr.clone()
        }
        SourceExpr::Log(x) => {
            if let SourceExpr::Exp(inner) = x.as_ref() {
                return *inner.clone();
            }
            expr.clone()
        }
        SourceExpr::Sin(x) => {
            if is_zero(x) {
                return SourceExpr::Int(0);
            }
            expr.clone()
        }
        SourceExpr::Cos(x) => {
            if is_zero(x) {
                return SourceExpr::Int(1);
            }
            expr.clone()
        }
        SourceExpr::Tan(x) => {
            if is_zero(x) {
                return SourceExpr::Int(0);
            }
            expr.clone()
        }
        SourceExpr::Sinh(x) => {
            if is_zero(x) {
                return SourceExpr::Int(0);
            }
            expr.clone()
        }
        SourceExpr::Cosh(x) => {
            if is_zero(x) {
                return SourceExpr::Int(1);
            }
            expr.clone()
        }
        SourceExpr::Tanh(x) => {
            if is_zero(x) {
                return SourceExpr::Int(0);
            }
            expr.clone()
        }
        SourceExpr::Sigmoid(x) => {
            if is_zero(x) {
                return SourceExpr::Rational(1, 2);
            }
            expr.clone()
        }
        SourceExpr::Softplus(x) => {
            if is_zero(x) {
                // ln(2) left symbolic: no exact rational constant replacement.
                return expr.clone();
            }
            expr.clone()
        }
        SourceExpr::Swish(x) => {
            if is_zero(x) {
                return SourceExpr::Int(0);
            }
            expr.clone()
        }
        SourceExpr::ReluSoft(x) => {
            if is_zero(x) {
                return expr.clone();
            }
            expr.clone()
        }
        SourceExpr::Elu(x, _) => {
            if is_zero(x) {
                return SourceExpr::Int(0);
            }
            expr.clone()
        }
        SourceExpr::LeakyRelu(x, _) => {
            if is_zero(x) {
                return SourceExpr::Int(0);
            }
            expr.clone()
        }
        SourceExpr::Softsign(x) => {
            if is_zero(x) {
                return SourceExpr::Int(0);
            }
            expr.clone()
        }
        SourceExpr::Mish(x) => {
            if is_zero(x) {
                return SourceExpr::Int(0);
            }
            expr.clone()
        }
        SourceExpr::Asin(_)
        | SourceExpr::Acos(_)
        | SourceExpr::Atan(_)
        | SourceExpr::Sqrt(_)
        | SourceExpr::GeluTanh(_) => expr.clone(),
        _ => expr.clone(),
    }
}

fn pow_i64(mut base: i64, mut exp: u32) -> i64 {
    let mut acc = 1i64;
    while exp > 0 {
        if (exp & 1) == 1 {
            acc = acc.saturating_mul(base);
        }
        base = base.saturating_mul(base);
        exp >>= 1;
    }
    acc
}

fn is_zero(expr: &SourceExpr) -> bool {
    matches!(as_rational(expr), Some((0, _)))
}

fn is_one(expr: &SourceExpr) -> bool {
    matches!(as_rational(expr), Some((1, 1)))
}

fn as_rational(expr: &SourceExpr) -> Option<(i64, i64)> {
    match expr {
        SourceExpr::Int(n) => Some((*n, 1)),
        SourceExpr::Rational(p, q) if *q != 0 => {
            let mut p = *p;
            let mut q = *q;
            if q < 0 {
                p = -p;
                q = -q;
            }
            let g = gcd_i64(p.abs(), q.abs());
            Some((p / g, q / g))
        }
        _ => None,
    }
}

fn from_rational(mut p: i64, mut q: i64) -> SourceExpr {
    if q == 0 {
        return SourceExpr::Rational(p, q);
    }
    if q < 0 {
        p = -p;
        q = -q;
    }
    let g = gcd_i64(p.abs(), q.abs());
    p /= g;
    q /= g;
    if q == 1 {
        SourceExpr::Int(p)
    } else {
        SourceExpr::Rational(p, q)
    }
}

fn gcd_i64(mut a: i64, mut b: i64) -> i64 {
    while b != 0 {
        let t = a % b;
        a = b;
        b = t;
    }
    if a == 0 {
        1
    } else {
        a.abs()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn optimize_removes_log_exp_pair() {
        let expr = SourceExpr::Log(Box::new(SourceExpr::Exp(Box::new(SourceExpr::var(0)))));
        let opt = optimize_for_lowering(&expr);
        assert_eq!(opt, SourceExpr::var(0));
    }

    #[test]
    fn optimize_drops_add_zero() {
        let expr = SourceExpr::Add(Box::new(SourceExpr::var(0)), Box::new(SourceExpr::Int(0)));
        let opt = optimize_for_lowering(&expr);
        assert_eq!(opt, SourceExpr::var(0));
    }

    #[test]
    fn optimized_cost_is_not_higher() {
        let expr = SourceExpr::Log(Box::new(SourceExpr::Exp(Box::new(SourceExpr::var(0)))));
        let opt = optimize_for_lowering(&expr);
        assert!(estimate_cost(&opt).score <= estimate_cost(&expr).score);
    }

    #[test]
    fn rewrite_preserves_sample_semantics() {
        let expr = SourceExpr::Sub(
            Box::new(SourceExpr::Add(
                Box::new(SourceExpr::var(0)),
                Box::new(SourceExpr::Int(0)),
            )),
            Box::new(SourceExpr::Log(Box::new(SourceExpr::Exp(Box::new(
                SourceExpr::var(1),
            ))))),
        );
        let opt = optimize_for_lowering(&expr);
        let vars = vec![Complex64::new(0.2, 0.1), Complex64::new(1.1, -0.2)];
        assert!(semantically_equivalent_on_sample(&expr, &opt, &vars, 1e-10));
    }
}
