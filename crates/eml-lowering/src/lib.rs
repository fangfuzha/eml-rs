#![no_std]
//! Standalone parser/lowering crate for EML.
//!
//! This crate is intentionally separated so parser/lowering logic can be used
//! without the full runtime stack. It is designed around `no_std + alloc`.

extern crate alloc;

use alloc::boxed::Box;
use alloc::format;
use alloc::string::{String, ToString};
use alloc::vec::Vec;
use core::fmt::{Display, Formatter};

use num_complex::Complex64;

/// Error type for parser and lowering flows.
#[derive(Debug, Clone, PartialEq)]
pub enum LoweringError {
    /// Mathematical domain error.
    Domain(&'static str),
    /// Variable index is out of bounds for provided input arity.
    MissingVariable { index: usize, arity: usize },
    /// Parsing failed.
    Parse(String),
    /// Feature is intentionally unsupported.
    Unsupported(&'static str),
    /// Integer/rational transformation overflow.
    Overflow(&'static str),
}

impl Display for LoweringError {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        match self {
            LoweringError::Domain(msg) => write!(f, "domain error: {msg}"),
            LoweringError::MissingVariable { index, arity } => {
                write!(f, "missing variable at index {index}, arity is {arity}")
            }
            LoweringError::Parse(msg) => write!(f, "parse error: {msg}"),
            LoweringError::Unsupported(msg) => write!(f, "unsupported: {msg}"),
            LoweringError::Overflow(msg) => write!(f, "overflow: {msg}"),
        }
    }
}

/// Source expression AST before lowering to pure EML.
#[derive(Debug, Clone, PartialEq)]
pub enum SourceExpr {
    /// Variable by index.
    Var(usize),
    /// Signed integer literal.
    Int(i64),
    /// Rational literal `p/q`.
    Rational(i64, i64),
    /// Euler's number `e`.
    ConstE,
    /// Imaginary unit `i`.
    ConstI,
    /// Pi.
    ConstPi,
    /// Unary negation.
    Neg(Box<SourceExpr>),
    /// Addition.
    Add(Box<SourceExpr>, Box<SourceExpr>),
    /// Subtraction.
    Sub(Box<SourceExpr>, Box<SourceExpr>),
    /// Multiplication.
    Mul(Box<SourceExpr>, Box<SourceExpr>),
    /// Division.
    Div(Box<SourceExpr>, Box<SourceExpr>),
    /// Power.
    Pow(Box<SourceExpr>, Box<SourceExpr>),
    /// Exponential.
    Exp(Box<SourceExpr>),
    /// Natural logarithm.
    Log(Box<SourceExpr>),
    /// Sine.
    Sin(Box<SourceExpr>),
    /// Cosine.
    Cos(Box<SourceExpr>),
    /// Tangent.
    Tan(Box<SourceExpr>),
    /// Hyperbolic sine.
    Sinh(Box<SourceExpr>),
    /// Hyperbolic cosine.
    Cosh(Box<SourceExpr>),
    /// Hyperbolic tangent.
    Tanh(Box<SourceExpr>),
    /// Inverse sine.
    Asin(Box<SourceExpr>),
    /// Inverse cosine.
    Acos(Box<SourceExpr>),
    /// Inverse tangent.
    Atan(Box<SourceExpr>),
    /// Square root.
    Sqrt(Box<SourceExpr>),
    /// Logistic sigmoid.
    Sigmoid(Box<SourceExpr>),
    /// Softplus.
    Softplus(Box<SourceExpr>),
    /// Swish: `x * sigmoid(x)`.
    Swish(Box<SourceExpr>),
    /// GELU tanh approximation.
    GeluTanh(Box<SourceExpr>),
    /// Smooth ReLU approximation using softplus.
    ReluSoft(Box<SourceExpr>),
    /// ELU with optional alpha parameter encoded as expression.
    Elu(Box<SourceExpr>, Box<SourceExpr>),
    /// Leaky ReLU with slope parameter encoded as expression.
    LeakyRelu(Box<SourceExpr>, Box<SourceExpr>),
    /// Softsign approximation.
    Softsign(Box<SourceExpr>),
    /// Mish: `x * tanh(softplus(x))`.
    Mish(Box<SourceExpr>),
}

impl SourceExpr {
    /// Convenience variable constructor.
    pub fn var(index: usize) -> Self {
        Self::Var(index)
    }
}

/// EML-only tree produced by lowering.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LoweredExpr {
    /// Constant `1`.
    One,
    /// Variable by zero-based index.
    Var(usize),
    /// EML node.
    Eml(Box<LoweredExpr>, Box<LoweredExpr>),
}

impl LoweredExpr {
    /// Convenience constructor for constant one.
    pub fn one() -> Self {
        Self::One
    }

    /// Convenience constructor for variables.
    pub fn var(index: usize) -> Self {
        Self::Var(index)
    }

    /// Convenience constructor for EML nodes.
    pub fn eml(lhs: LoweredExpr, rhs: LoweredExpr) -> Self {
        Self::Eml(Box::new(lhs), Box::new(rhs))
    }
}

/// Parses an infix expression string into [`SourceExpr`].
///
/// Examples:
/// - `sin(x0) + cos(x0)^2`
/// - `exp(x) - log(y)`
/// - `pow(x1, 3) / 2`
pub fn parse_source_expr(input: &str) -> Result<SourceExpr, LoweringError> {
    let tokens = lex(input)?;
    let mut parser = Parser::new(tokens);
    let expr = parser.parse_expr()?;
    if parser.peek().is_some() {
        return Err(LoweringError::Parse(
            "unexpected trailing tokens".to_string(),
        ));
    }
    Ok(expr)
}

/// Builds `logsumexp` template from a vector of logits.
///
/// Formula:
/// `logsumexp(z) = log(sum_j exp(z_j))`.
pub fn logsumexp_template(logits: &[SourceExpr]) -> Result<SourceExpr, LoweringError> {
    if logits.is_empty() {
        return Err(LoweringError::Domain("logits vector must not be empty"));
    }
    let mut sum: Option<SourceExpr> = None;
    for logit in logits {
        let e = SourceExpr::Exp(Box::new(logit.clone()));
        sum = Some(match sum {
            Some(prev) => SourceExpr::Add(Box::new(prev), Box::new(e)),
            None => e,
        });
    }
    Ok(SourceExpr::Log(Box::new(
        sum.expect("non-empty logits must produce sum"),
    )))
}

/// Builds vector softmax templates from logits.
///
/// For each `i`:
/// `softmax_i = exp(z_i) / sum_j exp(z_j)`.
pub fn softmax_template(logits: &[SourceExpr]) -> Result<Vec<SourceExpr>, LoweringError> {
    if logits.is_empty() {
        return Err(LoweringError::Domain("logits vector must not be empty"));
    }

    let mut exps = Vec::<SourceExpr>::with_capacity(logits.len());
    let mut sum: Option<SourceExpr> = None;
    for logit in logits {
        let e = SourceExpr::Exp(Box::new(logit.clone()));
        exps.push(e.clone());
        sum = Some(match sum {
            Some(prev) => SourceExpr::Add(Box::new(prev), Box::new(e)),
            None => e,
        });
    }

    let denom = sum.expect("non-empty logits must produce denominator");
    let mut out = Vec::<SourceExpr>::with_capacity(exps.len());
    for e in exps {
        out.push(SourceExpr::Div(Box::new(e), Box::new(denom.clone())));
    }
    Ok(out)
}

/// Builds one-hot cross-entropy template from logits and target index.
///
/// Numerically stable form:
/// `CE(z, t) = logsumexp(z) - z_t`.
pub fn cross_entropy_template(
    logits: &[SourceExpr],
    target_index: usize,
) -> Result<SourceExpr, LoweringError> {
    if logits.is_empty() {
        return Err(LoweringError::Domain("logits vector must not be empty"));
    }
    if target_index >= logits.len() {
        return Err(LoweringError::Domain(
            "target_index must be within logits range",
        ));
    }
    Ok(SourceExpr::Sub(
        Box::new(logsumexp_template(logits)?),
        Box::new(logits[target_index].clone()),
    ))
}

/// Builds one-hot label-smoothing cross-entropy template from logits.
///
/// Formula:
/// `LSCE(z, t, eps) = logsumexp(z) - ((1-eps) * z_t + eps * mean(z))`.
pub fn label_smoothing_cross_entropy_template(
    logits: &[SourceExpr],
    target_index: usize,
    epsilon: SourceExpr,
) -> Result<SourceExpr, LoweringError> {
    if logits.is_empty() {
        return Err(LoweringError::Domain("logits vector must not be empty"));
    }
    if target_index >= logits.len() {
        return Err(LoweringError::Domain(
            "target_index must be within logits range",
        ));
    }

    let lse = logsumexp_template(logits)?;
    let mut sum = src_zero();
    for logit in logits {
        sum = src_add(sum, logit.clone());
    }
    let class_count = i64::try_from(logits.len())
        .map_err(|_| LoweringError::Overflow("class count does not fit i64"))?;
    let mean_logit = src_div(sum, SourceExpr::Int(class_count));
    let one_minus_eps = src_sub(src_one(), epsilon.clone());
    let blended = src_add(
        src_mul(one_minus_eps, logits[target_index].clone()),
        src_mul(epsilon, mean_logit),
    );
    Ok(src_sub(lse, blended))
}

/// Builds focal-loss template from logits with `alpha = 1`.
///
/// Formula:
/// `FL(z, t) = (1 - p_t)^gamma * CE(z, t)`, where `p_t = softmax_t(z)`.
pub fn focal_loss_template(
    logits: &[SourceExpr],
    target_index: usize,
    gamma: SourceExpr,
) -> Result<SourceExpr, LoweringError> {
    focal_loss_template_with_alpha(logits, target_index, gamma, src_one())
}

/// Builds focal-loss template from logits with explicit `alpha` weight.
///
/// Formula:
/// `FL(z, t) = alpha * (1 - p_t)^gamma * CE(z, t)`.
pub fn focal_loss_template_with_alpha(
    logits: &[SourceExpr],
    target_index: usize,
    gamma: SourceExpr,
    alpha: SourceExpr,
) -> Result<SourceExpr, LoweringError> {
    if logits.is_empty() {
        return Err(LoweringError::Domain("logits vector must not be empty"));
    }
    if target_index >= logits.len() {
        return Err(LoweringError::Domain(
            "target_index must be within logits range",
        ));
    }

    let lse = logsumexp_template(logits)?;
    let z_t = logits[target_index].clone();
    let p_t = src_exp(src_sub(z_t.clone(), lse.clone()));
    let one_minus_pt = src_sub(src_one(), p_t);
    let modulating = src_pow(one_minus_pt, gamma);
    let ce = src_sub(lse, z_t);
    Ok(src_mul(alpha, src_mul(modulating, ce)))
}

/// Builds batched softmax templates from a batch of logits vectors.
///
/// Each entry in `batch_logits` represents one sample's logits.
pub fn batch_softmax_template(
    batch_logits: &[Vec<SourceExpr>],
) -> Result<Vec<Vec<SourceExpr>>, LoweringError> {
    if batch_logits.is_empty() {
        return Err(LoweringError::Domain("batch logits must not be empty"));
    }
    let mut out = Vec::with_capacity(batch_logits.len());
    for logits in batch_logits {
        out.push(softmax_template(logits)?);
    }
    Ok(out)
}

/// Builds batched one-hot cross-entropy templates.
///
/// `targets[i]` is the class index for `batch_logits[i]`.
pub fn batch_cross_entropy_template(
    batch_logits: &[Vec<SourceExpr>],
    targets: &[usize],
) -> Result<Vec<SourceExpr>, LoweringError> {
    if batch_logits.is_empty() {
        return Err(LoweringError::Domain("batch logits must not be empty"));
    }
    if batch_logits.len() != targets.len() {
        return Err(LoweringError::Domain(
            "batch logits and targets must have the same length",
        ));
    }

    let mut out = Vec::with_capacity(batch_logits.len());
    for (logits, target) in batch_logits.iter().zip(targets.iter().copied()) {
        out.push(cross_entropy_template(logits, target)?);
    }
    Ok(out)
}

/// Builds mean cross-entropy over a batch.
///
/// Formula:
/// `mean_ce = (1 / B) * sum_i CE(logits_i, target_i)`.
pub fn batch_cross_entropy_mean_template(
    batch_logits: &[Vec<SourceExpr>],
    targets: &[usize],
) -> Result<SourceExpr, LoweringError> {
    let losses = batch_cross_entropy_template(batch_logits, targets)?;
    mean_over_batch(losses, targets.len())
}

/// Builds batched one-hot label-smoothing cross-entropy templates.
pub fn batch_label_smoothing_cross_entropy_template(
    batch_logits: &[Vec<SourceExpr>],
    targets: &[usize],
    epsilon: SourceExpr,
) -> Result<Vec<SourceExpr>, LoweringError> {
    if batch_logits.is_empty() {
        return Err(LoweringError::Domain("batch logits must not be empty"));
    }
    if batch_logits.len() != targets.len() {
        return Err(LoweringError::Domain(
            "batch logits and targets must have the same length",
        ));
    }

    let mut out = Vec::with_capacity(batch_logits.len());
    for (logits, target) in batch_logits.iter().zip(targets.iter().copied()) {
        out.push(label_smoothing_cross_entropy_template(
            logits,
            target,
            epsilon.clone(),
        )?);
    }
    Ok(out)
}

/// Builds mean label-smoothing cross-entropy over a batch.
pub fn batch_label_smoothing_cross_entropy_mean_template(
    batch_logits: &[Vec<SourceExpr>],
    targets: &[usize],
    epsilon: SourceExpr,
) -> Result<SourceExpr, LoweringError> {
    let losses = batch_label_smoothing_cross_entropy_template(batch_logits, targets, epsilon)?;
    mean_over_batch(losses, targets.len())
}

/// Builds batched focal-loss templates with `alpha = 1`.
pub fn batch_focal_loss_template(
    batch_logits: &[Vec<SourceExpr>],
    targets: &[usize],
    gamma: SourceExpr,
) -> Result<Vec<SourceExpr>, LoweringError> {
    batch_focal_loss_template_with_alpha(batch_logits, targets, gamma, src_one())
}

/// Builds batched focal-loss templates with explicit `alpha`.
pub fn batch_focal_loss_template_with_alpha(
    batch_logits: &[Vec<SourceExpr>],
    targets: &[usize],
    gamma: SourceExpr,
    alpha: SourceExpr,
) -> Result<Vec<SourceExpr>, LoweringError> {
    if batch_logits.is_empty() {
        return Err(LoweringError::Domain("batch logits must not be empty"));
    }
    if batch_logits.len() != targets.len() {
        return Err(LoweringError::Domain(
            "batch logits and targets must have the same length",
        ));
    }

    let mut out = Vec::with_capacity(batch_logits.len());
    for (logits, target) in batch_logits.iter().zip(targets.iter().copied()) {
        out.push(focal_loss_template_with_alpha(
            logits,
            target,
            gamma.clone(),
            alpha.clone(),
        )?);
    }
    Ok(out)
}

/// Builds mean focal-loss over a batch with `alpha = 1`.
pub fn batch_focal_loss_mean_template(
    batch_logits: &[Vec<SourceExpr>],
    targets: &[usize],
    gamma: SourceExpr,
) -> Result<SourceExpr, LoweringError> {
    batch_focal_loss_mean_template_with_alpha(batch_logits, targets, gamma, src_one())
}

/// Builds mean focal-loss over a batch with explicit `alpha`.
pub fn batch_focal_loss_mean_template_with_alpha(
    batch_logits: &[Vec<SourceExpr>],
    targets: &[usize],
    gamma: SourceExpr,
    alpha: SourceExpr,
) -> Result<SourceExpr, LoweringError> {
    let losses = batch_focal_loss_template_with_alpha(batch_logits, targets, gamma, alpha)?;
    mean_over_batch(losses, targets.len())
}

fn mean_over_batch(losses: Vec<SourceExpr>, batch_len: usize) -> Result<SourceExpr, LoweringError> {
    let mut iter = losses.into_iter();
    let mut sum = iter
        .next()
        .ok_or(LoweringError::Domain("batch logits must not be empty"))?;
    for loss in iter {
        sum = src_add(sum, loss);
    }
    let batch_size = i64::try_from(batch_len)
        .map_err(|_| LoweringError::Overflow("batch size does not fit i64"))?;
    Ok(src_div(sum, SourceExpr::Int(batch_size)))
}

/// Symbolically differentiates a source expression with respect to `x{var_index}`.
///
/// The result is another [`SourceExpr`] and can be lowered/evaluated like any
/// other source expression.
pub fn symbolic_derivative(expr: &SourceExpr, var_index: usize) -> SourceExpr {
    simplify_source_expr(&symbolic_derivative_impl(expr, var_index))
}

/// Simplifies a source expression using local algebraic/constant-folding rules.
pub fn simplify_source_expr(expr: &SourceExpr) -> SourceExpr {
    match expr {
        SourceExpr::Var(index) => SourceExpr::Var(*index),
        SourceExpr::Int(n) => SourceExpr::Int(*n),
        SourceExpr::Rational(p, q) => match normalize_rational(*p as i128, *q as i128) {
            Some((num, den)) => {
                rational_expr_from_i128(num, den).unwrap_or(SourceExpr::Rational(*p, *q))
            }
            None => SourceExpr::Rational(*p, *q),
        },
        SourceExpr::ConstE => SourceExpr::ConstE,
        SourceExpr::ConstI => SourceExpr::ConstI,
        SourceExpr::ConstPi => SourceExpr::ConstPi,
        SourceExpr::Neg(x) => src_neg(simplify_source_expr(x)),
        SourceExpr::Add(a, b) => src_add(simplify_source_expr(a), simplify_source_expr(b)),
        SourceExpr::Sub(a, b) => src_sub(simplify_source_expr(a), simplify_source_expr(b)),
        SourceExpr::Mul(a, b) => src_mul(simplify_source_expr(a), simplify_source_expr(b)),
        SourceExpr::Div(a, b) => src_div(simplify_source_expr(a), simplify_source_expr(b)),
        SourceExpr::Pow(a, b) => src_pow(simplify_source_expr(a), simplify_source_expr(b)),
        SourceExpr::Exp(x) => src_exp(simplify_source_expr(x)),
        SourceExpr::Log(x) => src_log(simplify_source_expr(x)),
        SourceExpr::Sin(x) => SourceExpr::Sin(src_box(simplify_source_expr(x))),
        SourceExpr::Cos(x) => SourceExpr::Cos(src_box(simplify_source_expr(x))),
        SourceExpr::Tan(x) => SourceExpr::Tan(src_box(simplify_source_expr(x))),
        SourceExpr::Sinh(x) => SourceExpr::Sinh(src_box(simplify_source_expr(x))),
        SourceExpr::Cosh(x) => SourceExpr::Cosh(src_box(simplify_source_expr(x))),
        SourceExpr::Tanh(x) => SourceExpr::Tanh(src_box(simplify_source_expr(x))),
        SourceExpr::Asin(x) => SourceExpr::Asin(src_box(simplify_source_expr(x))),
        SourceExpr::Acos(x) => SourceExpr::Acos(src_box(simplify_source_expr(x))),
        SourceExpr::Atan(x) => SourceExpr::Atan(src_box(simplify_source_expr(x))),
        SourceExpr::Sqrt(x) => SourceExpr::Sqrt(src_box(simplify_source_expr(x))),
        SourceExpr::Sigmoid(x) => SourceExpr::Sigmoid(src_box(simplify_source_expr(x))),
        SourceExpr::Softplus(x) => SourceExpr::Softplus(src_box(simplify_source_expr(x))),
        SourceExpr::Swish(x) => SourceExpr::Swish(src_box(simplify_source_expr(x))),
        SourceExpr::GeluTanh(x) => SourceExpr::GeluTanh(src_box(simplify_source_expr(x))),
        SourceExpr::ReluSoft(x) => SourceExpr::ReluSoft(src_box(simplify_source_expr(x))),
        SourceExpr::Elu(x, alpha) => SourceExpr::Elu(
            src_box(simplify_source_expr(x)),
            src_box(simplify_source_expr(alpha)),
        ),
        SourceExpr::LeakyRelu(x, slope) => SourceExpr::LeakyRelu(
            src_box(simplify_source_expr(x)),
            src_box(simplify_source_expr(slope)),
        ),
        SourceExpr::Softsign(x) => SourceExpr::Softsign(src_box(simplify_source_expr(x))),
        SourceExpr::Mish(x) => SourceExpr::Mish(src_box(simplify_source_expr(x))),
    }
}

/// Returns total node count in a source expression tree.
pub fn source_expr_node_count(expr: &SourceExpr) -> usize {
    1 + match expr {
        SourceExpr::Var(_)
        | SourceExpr::Int(_)
        | SourceExpr::Rational(_, _)
        | SourceExpr::ConstE
        | SourceExpr::ConstI
        | SourceExpr::ConstPi => 0,
        SourceExpr::Neg(x)
        | SourceExpr::Exp(x)
        | SourceExpr::Log(x)
        | SourceExpr::Sin(x)
        | SourceExpr::Cos(x)
        | SourceExpr::Tan(x)
        | SourceExpr::Sinh(x)
        | SourceExpr::Cosh(x)
        | SourceExpr::Tanh(x)
        | SourceExpr::Asin(x)
        | SourceExpr::Acos(x)
        | SourceExpr::Atan(x)
        | SourceExpr::Sqrt(x)
        | SourceExpr::Sigmoid(x)
        | SourceExpr::Softplus(x)
        | SourceExpr::Swish(x)
        | SourceExpr::GeluTanh(x)
        | SourceExpr::ReluSoft(x)
        | SourceExpr::Softsign(x)
        | SourceExpr::Mish(x) => source_expr_node_count(x),
        SourceExpr::Add(a, b)
        | SourceExpr::Sub(a, b)
        | SourceExpr::Mul(a, b)
        | SourceExpr::Div(a, b)
        | SourceExpr::Pow(a, b)
        | SourceExpr::Elu(a, b)
        | SourceExpr::LeakyRelu(a, b) => source_expr_node_count(a) + source_expr_node_count(b),
    }
}

fn symbolic_derivative_impl(expr: &SourceExpr, var_index: usize) -> SourceExpr {
    match expr {
        SourceExpr::Var(index) => {
            if *index == var_index {
                src_one()
            } else {
                src_zero()
            }
        }
        SourceExpr::Int(_)
        | SourceExpr::Rational(_, _)
        | SourceExpr::ConstE
        | SourceExpr::ConstI
        | SourceExpr::ConstPi => src_zero(),
        SourceExpr::Neg(x) => src_neg(symbolic_derivative_impl(x, var_index)),
        SourceExpr::Add(a, b) => src_add(
            symbolic_derivative_impl(a, var_index),
            symbolic_derivative_impl(b, var_index),
        ),
        SourceExpr::Sub(a, b) => src_sub(
            symbolic_derivative_impl(a, var_index),
            symbolic_derivative_impl(b, var_index),
        ),
        SourceExpr::Mul(a, b) => {
            let da = symbolic_derivative_impl(a, var_index);
            let db = symbolic_derivative_impl(b, var_index);
            let av = (**a).clone();
            let bv = (**b).clone();
            src_add(src_mul(da, bv.clone()), src_mul(av, db))
        }
        SourceExpr::Div(a, b) => {
            let da = symbolic_derivative_impl(a, var_index);
            let db = symbolic_derivative_impl(b, var_index);
            let av = (**a).clone();
            let bv = (**b).clone();
            let numerator = src_sub(src_mul(da, bv.clone()), src_mul(av, db));
            let denominator = src_pow(bv, src_two());
            src_div(numerator, denominator)
        }
        SourceExpr::Pow(a, b) => {
            let da = symbolic_derivative_impl(a, var_index);
            let db = symbolic_derivative_impl(b, var_index);
            let av = (**a).clone();
            let bv = (**b).clone();
            if is_zero_expr(&db) {
                let b_minus_one = src_sub(bv.clone(), src_one());
                return src_mul(src_mul(bv, src_pow(av.clone(), b_minus_one)), da);
            }
            if is_zero_expr(&da) {
                return src_mul(src_mul(src_pow(av.clone(), bv.clone()), src_log(av)), db);
            }
            let lhs = src_mul(db, src_log(av.clone()));
            let rhs = src_mul(bv.clone(), src_div(da, av.clone()));
            src_mul(src_pow(av, bv), src_add(lhs, rhs))
        }
        SourceExpr::Exp(x) => {
            let dx = symbolic_derivative_impl(x, var_index);
            src_mul(src_exp((**x).clone()), dx)
        }
        SourceExpr::Log(x) => {
            let dx = symbolic_derivative_impl(x, var_index);
            src_div(dx, (**x).clone())
        }
        SourceExpr::Sin(x) => {
            let dx = symbolic_derivative_impl(x, var_index);
            src_mul(SourceExpr::Cos(src_box((**x).clone())), dx)
        }
        SourceExpr::Cos(x) => {
            let dx = symbolic_derivative_impl(x, var_index);
            src_mul(src_neg(SourceExpr::Sin(src_box((**x).clone()))), dx)
        }
        SourceExpr::Tan(x) => {
            let dx = symbolic_derivative_impl(x, var_index);
            let denom = src_pow(SourceExpr::Cos(src_box((**x).clone())), src_two());
            src_div(dx, denom)
        }
        SourceExpr::Sinh(x) => {
            let dx = symbolic_derivative_impl(x, var_index);
            src_mul(SourceExpr::Cosh(src_box((**x).clone())), dx)
        }
        SourceExpr::Cosh(x) => {
            let dx = symbolic_derivative_impl(x, var_index);
            src_mul(SourceExpr::Sinh(src_box((**x).clone())), dx)
        }
        SourceExpr::Tanh(x) => {
            let dx = symbolic_derivative_impl(x, var_index);
            let denom = src_pow(SourceExpr::Cosh(src_box((**x).clone())), src_two());
            src_div(dx, denom)
        }
        SourceExpr::Asin(x) => {
            let dx = symbolic_derivative_impl(x, var_index);
            let inner = src_sub(src_one(), src_pow((**x).clone(), src_two()));
            src_div(dx, SourceExpr::Sqrt(src_box(inner)))
        }
        SourceExpr::Acos(x) => {
            let dx = symbolic_derivative_impl(x, var_index);
            let inner = src_sub(src_one(), src_pow((**x).clone(), src_two()));
            src_neg(src_div(dx, SourceExpr::Sqrt(src_box(inner))))
        }
        SourceExpr::Atan(x) => {
            let dx = symbolic_derivative_impl(x, var_index);
            src_div(dx, src_add(src_one(), src_pow((**x).clone(), src_two())))
        }
        SourceExpr::Sqrt(x) => {
            let dx = symbolic_derivative_impl(x, var_index);
            let denom = src_mul(src_two(), SourceExpr::Sqrt(src_box((**x).clone())));
            src_div(dx, denom)
        }
        SourceExpr::Sigmoid(x) => {
            let dx = symbolic_derivative_impl(x, var_index);
            let sig = SourceExpr::Sigmoid(src_box((**x).clone()));
            src_mul(dx, src_mul(sig.clone(), src_sub(src_one(), sig)))
        }
        SourceExpr::Softplus(x) => {
            let dx = symbolic_derivative_impl(x, var_index);
            src_mul(dx, SourceExpr::Sigmoid(src_box((**x).clone())))
        }
        SourceExpr::Swish(x) => {
            let dx = symbolic_derivative_impl(x, var_index);
            let xv = (**x).clone();
            let sig = SourceExpr::Sigmoid(src_box(xv.clone()));
            let term = src_add(
                sig.clone(),
                src_mul(xv, src_mul(sig.clone(), src_sub(src_one(), sig))),
            );
            src_mul(dx, term)
        }
        SourceExpr::GeluTanh(x) => {
            let expanded = simplify_source_expr(&expand_gelu_tanh((**x).clone()));
            symbolic_derivative_impl(&expanded, var_index)
        }
        SourceExpr::ReluSoft(x) => {
            let expanded = SourceExpr::Softplus(src_box((**x).clone()));
            symbolic_derivative_impl(&expanded, var_index)
        }
        SourceExpr::Elu(x, alpha) => {
            let expanded = simplify_source_expr(&expand_elu((**x).clone(), (**alpha).clone()));
            symbolic_derivative_impl(&expanded, var_index)
        }
        SourceExpr::LeakyRelu(x, slope) => {
            let expanded =
                simplify_source_expr(&expand_leaky_relu((**x).clone(), (**slope).clone()));
            symbolic_derivative_impl(&expanded, var_index)
        }
        SourceExpr::Softsign(x) => {
            let expanded = simplify_source_expr(&expand_softsign((**x).clone()));
            symbolic_derivative_impl(&expanded, var_index)
        }
        SourceExpr::Mish(x) => {
            let expanded = simplify_source_expr(&expand_mish((**x).clone()));
            symbolic_derivative_impl(&expanded, var_index)
        }
    }
}

/// Evaluates a lowered EML tree directly using native complex math.
pub fn eval_lowered_expr_complex(
    expr: &LoweredExpr,
    vars: &[Complex64],
) -> Result<Complex64, LoweringError> {
    match expr {
        LoweredExpr::One => Ok(Complex64::new(1.0, 0.0)),
        LoweredExpr::Var(index) => {
            vars.get(*index)
                .copied()
                .ok_or(LoweringError::MissingVariable {
                    index: *index,
                    arity: vars.len(),
                })
        }
        LoweredExpr::Eml(lhs, rhs) => {
            let l = eval_lowered_expr_complex(lhs, vars)?;
            let r = eval_lowered_expr_complex(rhs, vars)?;
            Ok(l.exp() - r.ln())
        }
    }
}

/// De-lowers pure EML trees back into source-level primitives.
///
/// Each EML node is expanded as:
/// `eml(a, b) => exp(a) - log(b)`.
pub fn delower_to_source(expr: &LoweredExpr) -> SourceExpr {
    match expr {
        LoweredExpr::One => SourceExpr::Int(1),
        LoweredExpr::Var(index) => SourceExpr::Var(*index),
        LoweredExpr::Eml(lhs, rhs) => src_sub(
            SourceExpr::Exp(src_box(delower_to_source(lhs))),
            SourceExpr::Log(src_box(delower_to_source(rhs))),
        ),
    }
}

fn src_box(expr: SourceExpr) -> Box<SourceExpr> {
    Box::new(expr)
}

fn src_zero() -> SourceExpr {
    SourceExpr::Int(0)
}

fn src_one() -> SourceExpr {
    SourceExpr::Int(1)
}

fn src_two() -> SourceExpr {
    SourceExpr::Int(2)
}

fn src_exp(x: SourceExpr) -> SourceExpr {
    if is_zero_expr(&x) {
        return src_one();
    }
    if let SourceExpr::Log(inner) = x {
        return *inner;
    }
    SourceExpr::Exp(src_box(x))
}

fn src_log(x: SourceExpr) -> SourceExpr {
    if is_one_expr(&x) {
        return src_zero();
    }
    if let SourceExpr::Exp(inner) = x {
        return *inner;
    }
    SourceExpr::Log(src_box(x))
}

fn src_neg(x: SourceExpr) -> SourceExpr {
    if is_zero_expr(&x) {
        return src_zero();
    }
    if let Some((n, d)) = as_rational_const(&x) {
        if let Some(expr) = rational_expr_from_i128(-(n as i128), d as i128) {
            return expr;
        }
    }
    if let SourceExpr::Neg(inner) = x {
        return *inner;
    }
    SourceExpr::Neg(src_box(x))
}

fn src_add(a: SourceExpr, b: SourceExpr) -> SourceExpr {
    if is_zero_expr(&a) {
        return b;
    }
    if is_zero_expr(&b) {
        return a;
    }
    if let Some(expr) = try_fold_add(&a, &b) {
        return expr;
    }
    if a == b {
        return src_mul(src_two(), a);
    }
    SourceExpr::Add(src_box(a), src_box(b))
}

fn src_sub(a: SourceExpr, b: SourceExpr) -> SourceExpr {
    if is_zero_expr(&b) {
        return a;
    }
    if is_zero_expr(&a) {
        return src_neg(b);
    }
    if a == b {
        return src_zero();
    }
    if let Some(expr) = try_fold_sub(&a, &b) {
        return expr;
    }
    SourceExpr::Sub(src_box(a), src_box(b))
}

fn src_mul(a: SourceExpr, b: SourceExpr) -> SourceExpr {
    if is_zero_expr(&a) || is_zero_expr(&b) {
        return src_zero();
    }
    if is_one_expr(&a) {
        return b;
    }
    if is_one_expr(&b) {
        return a;
    }
    if is_minus_one_expr(&a) {
        return src_neg(b);
    }
    if is_minus_one_expr(&b) {
        return src_neg(a);
    }
    if let Some(expr) = try_fold_mul(&a, &b) {
        return expr;
    }
    SourceExpr::Mul(src_box(a), src_box(b))
}

fn src_div(a: SourceExpr, b: SourceExpr) -> SourceExpr {
    if is_zero_expr(&a) {
        return src_zero();
    }
    if is_one_expr(&b) {
        return a;
    }
    if is_minus_one_expr(&b) {
        return src_neg(a);
    }
    if let Some(expr) = try_fold_div(&a, &b) {
        return expr;
    }
    SourceExpr::Div(src_box(a), src_box(b))
}

fn src_pow(a: SourceExpr, b: SourceExpr) -> SourceExpr {
    if is_zero_expr(&b) {
        return src_one();
    }
    if is_one_expr(&b) {
        return a;
    }
    if is_one_expr(&a) {
        return src_one();
    }
    if is_zero_expr(&a) {
        if let Some((n, d)) = as_rational_const(&b) {
            if d == 1 && n > 0 {
                return src_zero();
            }
        }
    }
    if let Some(expr) = try_fold_pow(&a, &b) {
        return expr;
    }
    SourceExpr::Pow(src_box(a), src_box(b))
}

fn is_zero_expr(expr: &SourceExpr) -> bool {
    matches!(as_rational_const(expr), Some((0, _)))
}

fn is_one_expr(expr: &SourceExpr) -> bool {
    matches!(as_rational_const(expr), Some((1, 1)))
}

fn is_minus_one_expr(expr: &SourceExpr) -> bool {
    matches!(as_rational_const(expr), Some((-1, 1)))
}

fn as_rational_const(expr: &SourceExpr) -> Option<(i64, i64)> {
    match expr {
        SourceExpr::Int(n) => Some((*n, 1)),
        SourceExpr::Rational(p, q) => {
            let (num, den) = normalize_rational(*p as i128, *q as i128)?;
            let num_i64 = i64::try_from(num).ok()?;
            let den_i64 = i64::try_from(den).ok()?;
            Some((num_i64, den_i64))
        }
        _ => None,
    }
}

fn try_fold_add(a: &SourceExpr, b: &SourceExpr) -> Option<SourceExpr> {
    let (an, ad) = as_rational_const(a)?;
    let (bn, bd) = as_rational_const(b)?;
    rational_expr_from_i128(
        (an as i128) * (bd as i128) + (bn as i128) * (ad as i128),
        (ad as i128) * (bd as i128),
    )
}

fn try_fold_sub(a: &SourceExpr, b: &SourceExpr) -> Option<SourceExpr> {
    let (an, ad) = as_rational_const(a)?;
    let (bn, bd) = as_rational_const(b)?;
    rational_expr_from_i128(
        (an as i128) * (bd as i128) - (bn as i128) * (ad as i128),
        (ad as i128) * (bd as i128),
    )
}

fn try_fold_mul(a: &SourceExpr, b: &SourceExpr) -> Option<SourceExpr> {
    let (an, ad) = as_rational_const(a)?;
    let (bn, bd) = as_rational_const(b)?;
    rational_expr_from_i128((an as i128) * (bn as i128), (ad as i128) * (bd as i128))
}

fn try_fold_div(a: &SourceExpr, b: &SourceExpr) -> Option<SourceExpr> {
    let (an, ad) = as_rational_const(a)?;
    let (bn, bd) = as_rational_const(b)?;
    if bn == 0 {
        return None;
    }
    rational_expr_from_i128((an as i128) * (bd as i128), (ad as i128) * (bn as i128))
}

fn try_fold_pow(a: &SourceExpr, b: &SourceExpr) -> Option<SourceExpr> {
    let (base_n, base_d) = as_rational_const(a)?;
    let (exp_n, exp_d) = as_rational_const(b)?;
    if exp_d != 1 {
        return None;
    }

    let exp = exp_n;
    if exp == 0 {
        return Some(src_one());
    }
    if exp > 0 {
        let n = pow_i128_checked(base_n as i128, exp as u32)?;
        let d = pow_i128_checked(base_d as i128, exp as u32)?;
        return rational_expr_from_i128(n, d);
    }

    let abs_exp = (-exp) as u32;
    let n = pow_i128_checked(base_n as i128, abs_exp)?;
    let d = pow_i128_checked(base_d as i128, abs_exp)?;
    if n == 0 {
        return None;
    }
    rational_expr_from_i128(d, n)
}

fn normalize_rational(num: i128, den: i128) -> Option<(i128, i128)> {
    if den == 0 {
        return None;
    }
    let mut n = num;
    let mut d = den;
    if d < 0 {
        n = -n;
        d = -d;
    }
    let g = gcd_i128(n, d)?;
    Some((n / g, d / g))
}

fn rational_expr_from_i128(num: i128, den: i128) -> Option<SourceExpr> {
    let (n, d) = normalize_rational(num, den)?;
    if n < i64::MIN as i128 || n > i64::MAX as i128 {
        return None;
    }
    if d < i64::MIN as i128 || d > i64::MAX as i128 {
        return None;
    }
    let ni = n as i64;
    let di = d as i64;
    if di == 1 {
        Some(SourceExpr::Int(ni))
    } else {
        Some(SourceExpr::Rational(ni, di))
    }
}

fn gcd_i128(a: i128, b: i128) -> Option<i128> {
    let mut x = a.checked_abs()?;
    let mut y = b.checked_abs()?;
    if y == 0 {
        return Some(if x == 0 { 1 } else { x });
    }
    while y != 0 {
        let t = x % y;
        x = y;
        y = t;
    }
    Some(if x == 0 { 1 } else { x })
}

fn pow_i128_checked(mut base: i128, mut exp: u32) -> Option<i128> {
    let mut acc = 1i128;
    while exp > 0 {
        if (exp & 1) == 1 {
            acc = acc.checked_mul(base)?;
        }
        exp >>= 1;
        if exp > 0 {
            base = base.checked_mul(base)?;
        }
    }
    Some(acc)
}

fn expand_gelu_tanh(z: SourceExpr) -> SourceExpr {
    // 0.5*x*(1 + tanh(sqrt(2/pi)*(x + 0.044715*x^3)))
    let c = SourceExpr::Sqrt(src_box(src_div(SourceExpr::Int(2), SourceExpr::ConstPi)));
    let a = SourceExpr::Rational(44_715, 1_000_000);
    let x3 = src_mul(z.clone(), src_mul(z.clone(), z.clone()));
    let inner = src_add(z.clone(), src_mul(a, x3));
    let tanh_inner = SourceExpr::Tanh(src_box(src_mul(c, inner)));
    src_mul(
        SourceExpr::Rational(1, 2),
        src_mul(z, src_add(src_one(), tanh_inner)),
    )
}

fn expand_elu(z: SourceExpr, alpha: SourceExpr) -> SourceExpr {
    // x*sigmoid(8*x) + alpha*(exp(x)-1)*(1-sigmoid(8*x))
    let one = src_one();
    let gate = SourceExpr::Sigmoid(src_box(src_mul(SourceExpr::Int(8), z.clone())));
    let neg = src_mul(
        alpha,
        src_sub(SourceExpr::Exp(src_box(z.clone())), one.clone()),
    );
    src_add(src_mul(z, gate.clone()), src_mul(neg, src_sub(one, gate)))
}

fn expand_leaky_relu(z: SourceExpr, slope: SourceExpr) -> SourceExpr {
    // x * (sigmoid(8*x) + slope*(1-sigmoid(8*x)))
    let one = src_one();
    let gate = SourceExpr::Sigmoid(src_box(src_mul(SourceExpr::Int(8), z.clone())));
    let factor = src_add(gate.clone(), src_mul(slope, src_sub(one, gate)));
    src_mul(z, factor)
}

fn expand_softsign(z: SourceExpr) -> SourceExpr {
    // x / (1 + sqrt(x^2 + 1/100))
    let denom = src_add(
        src_one(),
        SourceExpr::Sqrt(src_box(src_add(
            src_mul(z.clone(), z.clone()),
            SourceExpr::Rational(1, 100),
        ))),
    );
    src_div(z, denom)
}

fn expand_mish(z: SourceExpr) -> SourceExpr {
    // x * tanh(softplus(x))
    src_mul(
        z.clone(),
        SourceExpr::Tanh(src_box(SourceExpr::Softplus(src_box(z)))),
    )
}

/// Evaluates source expression directly with native complex operators.
///
/// This is useful as a reference path for verification tests.
pub fn eval_source_expr_complex(
    expr: &SourceExpr,
    vars: &[Complex64],
) -> Result<Complex64, LoweringError> {
    match expr {
        SourceExpr::Var(index) => vars
            .get(*index)
            .copied()
            .ok_or(LoweringError::MissingVariable {
                index: *index,
                arity: vars.len(),
            }),
        SourceExpr::Int(n) => Ok(Complex64::new(*n as f64, 0.0)),
        SourceExpr::Rational(p, q) => {
            if *q == 0 {
                return Err(LoweringError::Domain(
                    "rational denominator must not be zero",
                ));
            }
            Ok(Complex64::new(*p as f64 / *q as f64, 0.0))
        }
        SourceExpr::ConstE => Ok(Complex64::new(core::f64::consts::E, 0.0)),
        SourceExpr::ConstI => Ok(Complex64::new(0.0, 1.0)),
        SourceExpr::ConstPi => Ok(Complex64::new(core::f64::consts::PI, 0.0)),
        SourceExpr::Neg(x) => Ok(-eval_source_expr_complex(x, vars)?),
        SourceExpr::Add(a, b) => {
            Ok(eval_source_expr_complex(a, vars)? + eval_source_expr_complex(b, vars)?)
        }
        SourceExpr::Sub(a, b) => {
            Ok(eval_source_expr_complex(a, vars)? - eval_source_expr_complex(b, vars)?)
        }
        SourceExpr::Mul(a, b) => {
            Ok(eval_source_expr_complex(a, vars)? * eval_source_expr_complex(b, vars)?)
        }
        SourceExpr::Div(a, b) => {
            Ok(eval_source_expr_complex(a, vars)? / eval_source_expr_complex(b, vars)?)
        }
        SourceExpr::Pow(a, b) => {
            let x = eval_source_expr_complex(a, vars)?;
            let y = eval_source_expr_complex(b, vars)?;
            Ok((x.ln() * y).exp())
        }
        SourceExpr::Exp(x) => Ok(eval_source_expr_complex(x, vars)?.exp()),
        SourceExpr::Log(x) => Ok(eval_source_expr_complex(x, vars)?.ln()),
        SourceExpr::Sin(x) => Ok(eval_source_expr_complex(x, vars)?.sin()),
        SourceExpr::Cos(x) => Ok(eval_source_expr_complex(x, vars)?.cos()),
        SourceExpr::Tan(x) => Ok(eval_source_expr_complex(x, vars)?.tan()),
        SourceExpr::Sinh(x) => Ok(eval_source_expr_complex(x, vars)?.sinh()),
        SourceExpr::Cosh(x) => Ok(eval_source_expr_complex(x, vars)?.cosh()),
        SourceExpr::Tanh(x) => Ok(eval_source_expr_complex(x, vars)?.tanh()),
        SourceExpr::Asin(x) => {
            let z = eval_source_expr_complex(x, vars)?;
            Ok(-Complex64::new(0.0, 1.0)
                * (Complex64::new(0.0, 1.0) * z + (Complex64::new(1.0, 0.0) - z * z).sqrt()).ln())
        }
        SourceExpr::Acos(x) => {
            let z = eval_source_expr_complex(x, vars)?;
            Ok(-Complex64::new(0.0, 1.0)
                * (z + Complex64::new(0.0, 1.0) * (Complex64::new(1.0, 0.0) - z * z).sqrt()).ln())
        }
        SourceExpr::Atan(x) => {
            let z = eval_source_expr_complex(x, vars)?;
            let i = Complex64::new(0.0, 1.0);
            let two = Complex64::new(2.0, 0.0);
            Ok((i / two)
                * ((Complex64::new(1.0, 0.0) - i * z).ln()
                    - (Complex64::new(1.0, 0.0) + i * z).ln()))
        }
        SourceExpr::Sqrt(x) => Ok(eval_source_expr_complex(x, vars)?.sqrt()),
        SourceExpr::Sigmoid(x) => {
            let z = eval_source_expr_complex(x, vars)?;
            Ok(Complex64::new(1.0, 0.0) / (Complex64::new(1.0, 0.0) + (-z).exp()))
        }
        SourceExpr::Softplus(x) => {
            let z = eval_source_expr_complex(x, vars)?;
            Ok((Complex64::new(1.0, 0.0) + z.exp()).ln())
        }
        SourceExpr::Swish(x) => {
            let z = eval_source_expr_complex(x, vars)?;
            let sigmoid = Complex64::new(1.0, 0.0) / (Complex64::new(1.0, 0.0) + (-z).exp());
            Ok(z * sigmoid)
        }
        SourceExpr::GeluTanh(x) => {
            // 0.5*x*(1 + tanh(sqrt(2/pi)*(x + 0.044715*x^3)))
            let z = eval_source_expr_complex(x, vars)?;
            let c = libm::sqrt(2.0 / core::f64::consts::PI);
            let inner = z + Complex64::new(0.044715, 0.0) * z * z * z;
            Ok(Complex64::new(0.5, 0.0)
                * z
                * (Complex64::new(1.0, 0.0) + (Complex64::new(c, 0.0) * inner).tanh()))
        }
        SourceExpr::ReluSoft(x) => {
            // softplus(x) as smooth ReLU proxy.
            let z = eval_source_expr_complex(x, vars)?;
            Ok((Complex64::new(1.0, 0.0) + z.exp()).ln())
        }
        SourceExpr::Elu(x, alpha) => {
            // Smooth ELU surrogate for pure-EML lowering:
            // x*sigmoid(beta*x) + alpha*(exp(x)-1)*(1-sigmoid(beta*x))
            let z = eval_source_expr_complex(x, vars)?;
            let a = eval_source_expr_complex(alpha, vars)?;
            let beta = Complex64::new(8.0, 0.0);
            let one = Complex64::new(1.0, 0.0);
            let gate = one / (one + (-(beta * z)).exp());
            let neg = a * (z.exp() - one);
            Ok(z * gate + neg * (one - gate))
        }
        SourceExpr::LeakyRelu(x, slope) => {
            // Smooth leaky-ReLU surrogate:
            // x * (sigmoid(beta*x) + slope*(1-sigmoid(beta*x)))
            let z = eval_source_expr_complex(x, vars)?;
            let a = eval_source_expr_complex(slope, vars)?;
            let beta = Complex64::new(8.0, 0.0);
            let one = Complex64::new(1.0, 0.0);
            let gate = one / (one + (-(beta * z)).exp());
            Ok(z * (gate + a * (one - gate)))
        }
        SourceExpr::Softsign(x) => {
            // Smooth abs surrogate: |x| ~= sqrt(x^2 + eps)
            let z = eval_source_expr_complex(x, vars)?;
            let one = Complex64::new(1.0, 0.0);
            let eps = Complex64::new(0.01, 0.0);
            Ok(z / (one + (z * z + eps).sqrt()))
        }
        SourceExpr::Mish(x) => {
            let z = eval_source_expr_complex(x, vars)?;
            let one = Complex64::new(1.0, 0.0);
            Ok(z * (one + z.exp()).ln().tanh())
        }
    }
}

/// Lowers a source expression into pure EML tree.
pub fn lower_to_eml(expr: &SourceExpr) -> Result<LoweredExpr, LoweringError> {
    match expr {
        SourceExpr::Var(index) => Ok(LoweredExpr::var(*index)),
        SourceExpr::Int(n) => eml_int(*n),
        SourceExpr::Rational(p, q) => eml_rational(*p, *q),
        SourceExpr::ConstE => Ok(eml_const_e()),
        SourceExpr::ConstI => eml_const_i(),
        SourceExpr::ConstPi => eml_const_pi(),
        SourceExpr::Neg(x) => Ok(eml_neg(lower_to_eml(x)?)),
        SourceExpr::Add(a, b) => Ok(eml_add(lower_to_eml(a)?, lower_to_eml(b)?)),
        SourceExpr::Sub(a, b) => Ok(eml_sub(lower_to_eml(a)?, lower_to_eml(b)?)),
        SourceExpr::Mul(a, b) => Ok(eml_mul(lower_to_eml(a)?, lower_to_eml(b)?)),
        SourceExpr::Div(a, b) => Ok(eml_div(lower_to_eml(a)?, lower_to_eml(b)?)),
        SourceExpr::Pow(a, b) => Ok(eml_pow(lower_to_eml(a)?, lower_to_eml(b)?)),
        SourceExpr::Exp(x) => Ok(eml_exp(lower_to_eml(x)?)),
        SourceExpr::Log(x) => Ok(eml_log(lower_to_eml(x)?)),
        SourceExpr::Sin(x) => eml_sin(lower_to_eml(x)?),
        SourceExpr::Cos(x) => eml_cos(lower_to_eml(x)?),
        SourceExpr::Tan(x) => eml_tan(lower_to_eml(x)?),
        SourceExpr::Sinh(x) => Ok(eml_sinh(lower_to_eml(x)?)),
        SourceExpr::Cosh(x) => Ok(eml_cosh(lower_to_eml(x)?)),
        SourceExpr::Tanh(x) => Ok(eml_tanh(lower_to_eml(x)?)),
        SourceExpr::Asin(x) => eml_asin(lower_to_eml(x)?),
        SourceExpr::Acos(x) => eml_acos(lower_to_eml(x)?),
        SourceExpr::Atan(x) => eml_atan(lower_to_eml(x)?),
        SourceExpr::Sqrt(x) => eml_sqrt(lower_to_eml(x)?),
        SourceExpr::Sigmoid(x) => Ok(eml_sigmoid(lower_to_eml(x)?)),
        SourceExpr::Softplus(x) => Ok(eml_softplus(lower_to_eml(x)?)),
        SourceExpr::Swish(x) => Ok(eml_swish(lower_to_eml(x)?)),
        SourceExpr::GeluTanh(x) => eml_gelu_tanh(lower_to_eml(x)?),
        SourceExpr::ReluSoft(x) => Ok(eml_relu_soft(lower_to_eml(x)?)),
        SourceExpr::Elu(x, alpha) => eml_elu(lower_to_eml(x)?, lower_to_eml(alpha)?),
        SourceExpr::LeakyRelu(x, slope) => eml_leaky_relu(lower_to_eml(x)?, lower_to_eml(slope)?),
        SourceExpr::Softsign(x) => eml_softsign(lower_to_eml(x)?),
        SourceExpr::Mish(x) => Ok(eml_mish(lower_to_eml(x)?)),
    }
}

fn eml_exp(z: LoweredExpr) -> LoweredExpr {
    LoweredExpr::eml(z, LoweredExpr::one())
}

fn eml_log(z: LoweredExpr) -> LoweredExpr {
    // ln(x) = eml(1, eml(eml(1, x), 1))
    LoweredExpr::eml(
        LoweredExpr::one(),
        LoweredExpr::eml(LoweredExpr::eml(LoweredExpr::one(), z), LoweredExpr::one()),
    )
}

fn eml_zero() -> LoweredExpr {
    eml_sub(LoweredExpr::one(), LoweredExpr::one())
}

fn eml_sub(a: LoweredExpr, b: LoweredExpr) -> LoweredExpr {
    LoweredExpr::eml(eml_log(a), eml_exp(b))
}

fn eml_neg(z: LoweredExpr) -> LoweredExpr {
    // -z = (1 - z) - 1
    eml_sub(eml_sub(LoweredExpr::one(), z), LoweredExpr::one())
}

fn eml_add(a: LoweredExpr, b: LoweredExpr) -> LoweredExpr {
    eml_sub(a, eml_neg(b))
}

fn eml_inv(z: LoweredExpr) -> LoweredExpr {
    eml_exp(eml_neg(eml_log(z)))
}

fn eml_mul(a: LoweredExpr, b: LoweredExpr) -> LoweredExpr {
    eml_exp(eml_add(eml_log(a), eml_log(b)))
}

fn eml_div(a: LoweredExpr, b: LoweredExpr) -> LoweredExpr {
    eml_mul(a, eml_inv(b))
}

fn eml_pow(a: LoweredExpr, b: LoweredExpr) -> LoweredExpr {
    eml_exp(eml_mul(b, eml_log(a)))
}

fn eml_const_e() -> LoweredExpr {
    eml_exp(LoweredExpr::one())
}

fn eml_const_i() -> Result<LoweredExpr, LoweringError> {
    // i = -exp(log(-1)/2)
    let minus_one = eml_neg(LoweredExpr::one());
    let two = eml_int(2)?;
    Ok(eml_neg(eml_exp(eml_div(eml_log(minus_one), two))))
}

fn eml_const_pi() -> Result<LoweredExpr, LoweringError> {
    // pi = i * log(-1)
    let i_expr = eml_const_i()?;
    let minus_one = eml_neg(LoweredExpr::one());
    Ok(eml_mul(i_expr, eml_log(minus_one)))
}

fn eml_sin(z: LoweredExpr) -> Result<LoweredExpr, LoweringError> {
    // sin(z) = (exp(i z) - exp(-i z)) / (2 i)
    let i_expr = eml_const_i()?;
    let two = eml_int(2)?;
    let iz = eml_mul(i_expr.clone(), z);
    let numerator = eml_sub(eml_exp(iz.clone()), eml_exp(eml_neg(iz)));
    let denominator = eml_mul(two, i_expr);
    Ok(eml_div(numerator, denominator))
}

fn eml_cos(z: LoweredExpr) -> Result<LoweredExpr, LoweringError> {
    // cos(z) = (exp(i z) + exp(-i z)) / 2
    let i_expr = eml_const_i()?;
    let two = eml_int(2)?;
    let iz = eml_mul(i_expr, z);
    let numerator = eml_add(eml_exp(iz.clone()), eml_exp(eml_neg(iz)));
    Ok(eml_div(numerator, two))
}

fn eml_tan(z: LoweredExpr) -> Result<LoweredExpr, LoweringError> {
    Ok(eml_div(eml_sin(z.clone())?, eml_cos(z)?))
}

fn eml_sinh(z: LoweredExpr) -> LoweredExpr {
    // sinh(z) = (exp(z) - exp(-z))/2
    let two = eml_add(LoweredExpr::one(), LoweredExpr::one());
    eml_div(eml_sub(eml_exp(z.clone()), eml_exp(eml_neg(z))), two)
}

fn eml_cosh(z: LoweredExpr) -> LoweredExpr {
    // cosh(z) = (exp(z) + exp(-z))/2
    let two = eml_add(LoweredExpr::one(), LoweredExpr::one());
    eml_div(eml_add(eml_exp(z.clone()), eml_exp(eml_neg(z))), two)
}

fn eml_tanh(z: LoweredExpr) -> LoweredExpr {
    eml_div(eml_sinh(z.clone()), eml_cosh(z))
}

fn eml_sqrt(z: LoweredExpr) -> Result<LoweredExpr, LoweringError> {
    Ok(eml_pow(z, eml_rational(1, 2)?))
}

fn eml_asin(z: LoweredExpr) -> Result<LoweredExpr, LoweringError> {
    // asin(z) = -i*log(i*z + sqrt(1-z^2))
    let i = eml_const_i()?;
    let one = LoweredExpr::one();
    let zz = eml_mul(z.clone(), z.clone());
    let root = eml_sqrt(eml_sub(one, zz))?;
    let inside = eml_add(eml_mul(i.clone(), z), root);
    Ok(eml_mul(eml_neg(i), eml_log(inside)))
}

fn eml_acos(z: LoweredExpr) -> Result<LoweredExpr, LoweringError> {
    // acos(z) = -i*log(z + i*sqrt(1-z^2))
    let i = eml_const_i()?;
    let one = LoweredExpr::one();
    let zz = eml_mul(z.clone(), z.clone());
    let root = eml_sqrt(eml_sub(one, zz))?;
    let inside = eml_add(z, eml_mul(i.clone(), root));
    Ok(eml_mul(eml_neg(i), eml_log(inside)))
}

fn eml_atan(z: LoweredExpr) -> Result<LoweredExpr, LoweringError> {
    // atan(z) = (i/2)*(log(1-i z) - log(1+i z))
    let i = eml_const_i()?;
    let half_i = eml_div(i.clone(), eml_int(2)?);
    let one = LoweredExpr::one();
    let term_a = eml_log(eml_sub(one.clone(), eml_mul(i.clone(), z.clone())));
    let term_b = eml_log(eml_add(one, eml_mul(i, z)));
    Ok(eml_mul(half_i, eml_sub(term_a, term_b)))
}

fn eml_sigmoid(z: LoweredExpr) -> LoweredExpr {
    // sigmoid(z) = 1 / (1 + exp(-z))
    eml_inv(eml_add(LoweredExpr::one(), eml_exp(eml_neg(z))))
}

fn eml_softplus(z: LoweredExpr) -> LoweredExpr {
    // softplus(z) = log(1 + exp(z))
    eml_log(eml_add(LoweredExpr::one(), eml_exp(z)))
}

fn eml_swish(z: LoweredExpr) -> LoweredExpr {
    // swish(z) = z * sigmoid(z)
    eml_mul(z.clone(), eml_sigmoid(z))
}

fn eml_relu_soft(z: LoweredExpr) -> LoweredExpr {
    // Smooth ReLU proxy based on softplus.
    eml_softplus(z)
}

fn eml_beta_eight() -> LoweredExpr {
    // 8 as a small exact integer tree without fallible conversions.
    let two = eml_add(LoweredExpr::one(), LoweredExpr::one());
    let four = eml_add(two.clone(), two);
    eml_add(four.clone(), four)
}

fn eml_elu(z: LoweredExpr, alpha: LoweredExpr) -> Result<LoweredExpr, LoweringError> {
    // Smooth ELU surrogate:
    // x*sigmoid(beta*x) + alpha*(exp(x)-1)*(1-sigmoid(beta*x))
    let one = LoweredExpr::one();
    let beta = eml_beta_eight();
    let gate = eml_sigmoid(eml_mul(beta, z.clone()));
    let neg = eml_mul(alpha, eml_sub(eml_exp(z.clone()), one.clone()));
    Ok(eml_add(
        eml_mul(z, gate.clone()),
        eml_mul(neg, eml_sub(one, gate)),
    ))
}

fn eml_leaky_relu(z: LoweredExpr, slope: LoweredExpr) -> Result<LoweredExpr, LoweringError> {
    // Smooth leaky-ReLU surrogate:
    // x * (sigmoid(beta*x) + slope*(1-sigmoid(beta*x)))
    let one = LoweredExpr::one();
    let beta = eml_beta_eight();
    let gate = eml_sigmoid(eml_mul(beta, z.clone()));
    let factor = eml_add(gate.clone(), eml_mul(slope, eml_sub(one, gate)));
    Ok(eml_mul(z, factor))
}

fn eml_softsign(z: LoweredExpr) -> Result<LoweredExpr, LoweringError> {
    // Smooth abs surrogate: |x| ~= sqrt(x^2 + eps), eps=1/100.
    let one = LoweredExpr::one();
    let eps = eml_rational(1, 100)?;
    let denom = eml_add(one, eml_sqrt(eml_add(eml_mul(z.clone(), z.clone()), eps))?);
    Ok(eml_div(z, denom))
}

fn eml_mish(z: LoweredExpr) -> LoweredExpr {
    // mish(x) = x * tanh(softplus(x))
    eml_mul(z.clone(), eml_tanh(eml_softplus(z)))
}

fn eml_gelu_tanh(z: LoweredExpr) -> Result<LoweredExpr, LoweringError> {
    // 0.5*x*(1+tanh(sqrt(2/pi)*(x+0.044715*x^3)))
    let half = eml_rational(1, 2)?;
    let c = eml_approx_real(libm::sqrt(2.0 / core::f64::consts::PI), 1_000)?;
    let a = eml_approx_real(0.044_715, 1_000)?;
    let x3 = eml_mul(z.clone(), eml_mul(z.clone(), z.clone()));
    let inner_poly = eml_add(z.clone(), eml_mul(a, x3));
    let t = eml_tanh(eml_mul(c, inner_poly));
    Ok(eml_mul(half, eml_mul(z, eml_add(LoweredExpr::one(), t))))
}

fn eml_int(n: i64) -> Result<LoweredExpr, LoweringError> {
    if n == 1 {
        return Ok(LoweredExpr::one());
    }
    if n == 0 {
        return Ok(eml_zero());
    }
    if n < 0 {
        return Ok(eml_neg(eml_int(-n)?));
    }

    let mut acc: Option<LoweredExpr> = None;
    let mut term = LoweredExpr::one();
    let mut k = n as u64;
    while k > 0 {
        if (k & 1) == 1 {
            acc = Some(match acc {
                Some(prev) => eml_add(prev, term.clone()),
                None => term.clone(),
            });
        }
        term = eml_add(term.clone(), term);
        k >>= 1;
    }
    Ok(acc.expect("positive integer decomposition must produce a value"))
}

fn eml_rational(p: i64, q: i64) -> Result<LoweredExpr, LoweringError> {
    if q == 0 {
        return Err(LoweringError::Domain(
            "rational denominator must not be zero",
        ));
    }
    if q == 1 {
        return eml_int(p);
    }

    let sign = if p < 0 { -1 } else { 1 };
    let num = eml_int(p.abs())?;
    let den = eml_int(q.abs())?;
    let val = eml_mul(num, eml_inv(den));
    Ok(if sign < 0 { eml_neg(val) } else { val })
}

fn eml_approx_real(value: f64, denom: i64) -> Result<LoweredExpr, LoweringError> {
    if !value.is_finite() {
        return Err(LoweringError::Domain(
            "non-finite real constant is not supported",
        ));
    }
    if denom <= 0 {
        return Err(LoweringError::Domain(
            "approximation denominator must be positive",
        ));
    }
    let scaled = value * denom as f64;
    let rounded = if scaled >= 0.0 {
        libm::floor(scaled + 0.5)
    } else {
        libm::ceil(scaled - 0.5)
    };
    if rounded < i64::MIN as f64 || rounded > i64::MAX as f64 {
        return Err(LoweringError::Overflow("constant approximation overflow"));
    }
    eml_rational(rounded as i64, denom)
}

#[derive(Debug, Clone, PartialEq)]
enum LexTok {
    Number(String),
    Ident(String),
    Plus,
    Minus,
    Star,
    Slash,
    Caret,
    LParen,
    RParen,
    Comma,
}

fn lex(input: &str) -> Result<Vec<LexTok>, LoweringError> {
    let chars: Vec<char> = input.chars().collect();
    let mut i = 0usize;
    let mut tokens = Vec::<LexTok>::new();

    while i < chars.len() {
        let c = chars[i];
        if c.is_whitespace() {
            i += 1;
            continue;
        }

        if c.is_ascii_digit() || c == '.' {
            let start = i;
            let mut seen_dot = c == '.';
            i += 1;
            while i < chars.len() {
                let d = chars[i];
                if d.is_ascii_digit() {
                    i += 1;
                } else if d == '.' && !seen_dot {
                    seen_dot = true;
                    i += 1;
                } else {
                    break;
                }
            }
            let text: String = chars[start..i].iter().collect();
            if text == "." {
                return Err(LoweringError::Parse(
                    "invalid numeric literal '.'".to_string(),
                ));
            }
            tokens.push(LexTok::Number(text));
            continue;
        }

        if c.is_ascii_alphabetic() || c == '_' {
            let start = i;
            i += 1;
            while i < chars.len() && (chars[i].is_ascii_alphanumeric() || chars[i] == '_') {
                i += 1;
            }
            let text: String = chars[start..i].iter().collect();
            tokens.push(LexTok::Ident(text));
            continue;
        }

        let tok = match c {
            '+' => LexTok::Plus,
            '-' => LexTok::Minus,
            '*' => LexTok::Star,
            '/' => LexTok::Slash,
            '^' => LexTok::Caret,
            '(' => LexTok::LParen,
            ')' => LexTok::RParen,
            ',' => LexTok::Comma,
            _ => {
                return Err(LoweringError::Parse(format!(
                    "unexpected character '{c}' at byte {i}"
                )))
            }
        };
        tokens.push(tok);
        i += 1;
    }

    Ok(tokens)
}

struct Parser {
    tokens: Vec<LexTok>,
    pos: usize,
}

impl Parser {
    fn new(tokens: Vec<LexTok>) -> Self {
        Self { tokens, pos: 0 }
    }

    fn peek(&self) -> Option<&LexTok> {
        self.tokens.get(self.pos)
    }

    fn bump(&mut self) -> Option<LexTok> {
        let tok = self.tokens.get(self.pos).cloned();
        self.pos += usize::from(tok.is_some());
        tok
    }

    fn parse_expr(&mut self) -> Result<SourceExpr, LoweringError> {
        self.parse_add_sub()
    }

    fn parse_add_sub(&mut self) -> Result<SourceExpr, LoweringError> {
        let mut lhs = self.parse_mul_div()?;
        loop {
            match self.peek() {
                Some(LexTok::Plus) => {
                    self.bump();
                    let rhs = self.parse_mul_div()?;
                    lhs = SourceExpr::Add(Box::new(lhs), Box::new(rhs));
                }
                Some(LexTok::Minus) => {
                    self.bump();
                    let rhs = self.parse_mul_div()?;
                    lhs = SourceExpr::Sub(Box::new(lhs), Box::new(rhs));
                }
                _ => break,
            }
        }
        Ok(lhs)
    }

    fn parse_mul_div(&mut self) -> Result<SourceExpr, LoweringError> {
        let mut lhs = self.parse_pow()?;
        loop {
            match self.peek() {
                Some(LexTok::Star) => {
                    self.bump();
                    let rhs = self.parse_pow()?;
                    lhs = SourceExpr::Mul(Box::new(lhs), Box::new(rhs));
                }
                Some(LexTok::Slash) => {
                    self.bump();
                    let rhs = self.parse_pow()?;
                    lhs = SourceExpr::Div(Box::new(lhs), Box::new(rhs));
                }
                _ => break,
            }
        }
        Ok(lhs)
    }

    fn parse_pow(&mut self) -> Result<SourceExpr, LoweringError> {
        let lhs = self.parse_unary()?;
        if matches!(self.peek(), Some(LexTok::Caret)) {
            self.bump();
            let rhs = self.parse_pow()?;
            return Ok(SourceExpr::Pow(Box::new(lhs), Box::new(rhs)));
        }
        Ok(lhs)
    }

    fn parse_unary(&mut self) -> Result<SourceExpr, LoweringError> {
        if matches!(self.peek(), Some(LexTok::Minus)) {
            self.bump();
            return Ok(SourceExpr::Neg(Box::new(self.parse_unary()?)));
        }
        self.parse_primary()
    }

    fn parse_primary(&mut self) -> Result<SourceExpr, LoweringError> {
        match self.bump() {
            Some(LexTok::Number(n)) => parse_number_literal(&n),
            Some(LexTok::Ident(id)) => {
                if matches!(self.peek(), Some(LexTok::LParen)) {
                    self.bump(); // (
                    let mut args = Vec::<SourceExpr>::new();
                    if !matches!(self.peek(), Some(LexTok::RParen)) {
                        loop {
                            args.push(self.parse_expr()?);
                            if matches!(self.peek(), Some(LexTok::Comma)) {
                                self.bump();
                            } else {
                                break;
                            }
                        }
                    }
                    if !matches!(self.bump(), Some(LexTok::RParen)) {
                        return Err(LoweringError::Parse("missing ')'".to_string()));
                    }
                    parse_function_call(&id, args)
                } else {
                    parse_identifier_atom(&id)
                }
            }
            Some(LexTok::LParen) => {
                let e = self.parse_expr()?;
                if !matches!(self.bump(), Some(LexTok::RParen)) {
                    return Err(LoweringError::Parse("missing ')'".to_string()));
                }
                Ok(e)
            }
            Some(tok) => Err(LoweringError::Parse(format!("unexpected token: {tok:?}"))),
            None => Err(LoweringError::Parse("unexpected end of input".to_string())),
        }
    }
}

fn parse_identifier_atom(id: &str) -> Result<SourceExpr, LoweringError> {
    let l = id.to_ascii_lowercase();
    match l.as_str() {
        "x" => Ok(SourceExpr::Var(0)),
        "y" => Ok(SourceExpr::Var(1)),
        "e" => Ok(SourceExpr::ConstE),
        "pi" => Ok(SourceExpr::ConstPi),
        "i" => Ok(SourceExpr::ConstI),
        "one" => Ok(SourceExpr::Int(1)),
        _ => {
            if let Some(rest) = l.strip_prefix('x') {
                if !rest.is_empty() {
                    let idx = rest.parse::<usize>().map_err(|_| {
                        LoweringError::Parse(format!("invalid variable identifier '{id}'"))
                    })?;
                    return Ok(SourceExpr::Var(idx));
                }
            }
            Err(LoweringError::Parse(format!("unknown identifier '{id}'")))
        }
    }
}

fn parse_function_call(name: &str, args: Vec<SourceExpr>) -> Result<SourceExpr, LoweringError> {
    let l = name.to_ascii_lowercase();
    match (l.as_str(), args.len()) {
        ("exp", 1) => Ok(SourceExpr::Exp(Box::new(args[0].clone()))),
        ("log", 1) | ("ln", 1) => Ok(SourceExpr::Log(Box::new(args[0].clone()))),
        ("sin", 1) => Ok(SourceExpr::Sin(Box::new(args[0].clone()))),
        ("cos", 1) => Ok(SourceExpr::Cos(Box::new(args[0].clone()))),
        ("tan", 1) => Ok(SourceExpr::Tan(Box::new(args[0].clone()))),
        ("sinh", 1) => Ok(SourceExpr::Sinh(Box::new(args[0].clone()))),
        ("cosh", 1) => Ok(SourceExpr::Cosh(Box::new(args[0].clone()))),
        ("tanh", 1) => Ok(SourceExpr::Tanh(Box::new(args[0].clone()))),
        ("asin", 1) => Ok(SourceExpr::Asin(Box::new(args[0].clone()))),
        ("acos", 1) => Ok(SourceExpr::Acos(Box::new(args[0].clone()))),
        ("atan", 1) => Ok(SourceExpr::Atan(Box::new(args[0].clone()))),
        ("sqrt", 1) => Ok(SourceExpr::Sqrt(Box::new(args[0].clone()))),
        ("sigmoid", 1) => Ok(SourceExpr::Sigmoid(Box::new(args[0].clone()))),
        ("softplus", 1) => Ok(SourceExpr::Softplus(Box::new(args[0].clone()))),
        ("swish", 1) => Ok(SourceExpr::Swish(Box::new(args[0].clone()))),
        ("gelu", 1) | ("gelu_tanh", 1) => Ok(SourceExpr::GeluTanh(Box::new(args[0].clone()))),
        ("relu", 1) | ("relu_soft", 1) => Ok(SourceExpr::ReluSoft(Box::new(args[0].clone()))),
        ("elu", 1) => Ok(SourceExpr::Elu(
            Box::new(args[0].clone()),
            Box::new(SourceExpr::Int(1)),
        )),
        ("elu", 2) => Ok(SourceExpr::Elu(
            Box::new(args[0].clone()),
            Box::new(args[1].clone()),
        )),
        ("leaky_relu", 1) | ("lrelu", 1) => Ok(SourceExpr::LeakyRelu(
            Box::new(args[0].clone()),
            Box::new(SourceExpr::Rational(1, 100)),
        )),
        ("leaky_relu", 2) | ("lrelu", 2) => Ok(SourceExpr::LeakyRelu(
            Box::new(args[0].clone()),
            Box::new(args[1].clone()),
        )),
        ("softsign", 1) => Ok(SourceExpr::Softsign(Box::new(args[0].clone()))),
        ("mish", 1) => Ok(SourceExpr::Mish(Box::new(args[0].clone()))),
        ("pow", 2) => Ok(SourceExpr::Pow(
            Box::new(args[0].clone()),
            Box::new(args[1].clone()),
        )),
        ("add", 2) | ("plus", 2) => Ok(SourceExpr::Add(
            Box::new(args[0].clone()),
            Box::new(args[1].clone()),
        )),
        ("sub", 2) | ("subtract", 2) => Ok(SourceExpr::Sub(
            Box::new(args[0].clone()),
            Box::new(args[1].clone()),
        )),
        ("mul", 2) | ("times", 2) => Ok(SourceExpr::Mul(
            Box::new(args[0].clone()),
            Box::new(args[1].clone()),
        )),
        ("div", 2) | ("divide", 2) => Ok(SourceExpr::Div(
            Box::new(args[0].clone()),
            Box::new(args[1].clone()),
        )),
        _ => Err(LoweringError::Parse(format!(
            "unsupported function call '{name}' with {} args",
            args.len()
        ))),
    }
}

fn parse_number_literal(text: &str) -> Result<SourceExpr, LoweringError> {
    if text.contains('.') {
        let (p, q) = decimal_to_rational(text)?;
        if q == 1 {
            Ok(SourceExpr::Int(p))
        } else {
            Ok(SourceExpr::Rational(p, q))
        }
    } else {
        let n = text
            .parse::<i64>()
            .map_err(|_| LoweringError::Parse(format!("invalid integer literal '{text}'")))?;
        Ok(SourceExpr::Int(n))
    }
}

fn decimal_to_rational(text: &str) -> Result<(i64, i64), LoweringError> {
    let parts: Vec<&str> = text.split('.').collect();
    if parts.len() != 2 {
        return Err(LoweringError::Parse(format!(
            "invalid decimal literal '{text}'"
        )));
    }
    let int_part = if parts[0].is_empty() { "0" } else { parts[0] };
    let frac_part = parts[1];
    let frac_len = frac_part.len() as u32;
    let den = 10i64
        .checked_pow(frac_len)
        .ok_or(LoweringError::Overflow("decimal scale overflow"))?;
    let int_num = int_part
        .parse::<i64>()
        .map_err(|_| LoweringError::Parse(format!("invalid decimal literal '{text}'")))?;
    let frac_num = if frac_part.is_empty() {
        0i64
    } else {
        frac_part
            .parse::<i64>()
            .map_err(|_| LoweringError::Parse(format!("invalid decimal literal '{text}'")))?
    };
    let mut num = int_num
        .checked_mul(den)
        .and_then(|v| v.checked_add(frac_num))
        .ok_or(LoweringError::Overflow("decimal numerator overflow"))?;
    let mut q = den;
    let g = gcd_i64(num.abs(), q.abs());
    num /= g;
    q /= g;
    Ok((num, q))
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
extern crate std;

#[cfg(test)]
mod tests {
    use alloc::vec;

    use super::*;

    fn finite_diff_real(expr: &SourceExpr, vars: &[Complex64], var_index: usize, h: f64) -> f64 {
        let mut plus = vars.to_vec();
        let mut minus = vars.to_vec();
        plus[var_index].re += h;
        minus[var_index].re -= h;
        let f_plus = eval_source_expr_complex(expr, &plus).unwrap();
        let f_minus = eval_source_expr_complex(expr, &minus).unwrap();
        ((f_plus - f_minus) / Complex64::new(2.0 * h, 0.0)).re
    }

    #[test]
    fn parse_basic_infix() {
        let expr = parse_source_expr("sin(x0) + cos(x0)^2").unwrap();
        match expr {
            SourceExpr::Add(_, _) => {}
            _ => panic!("unexpected parse tree: {expr:?}"),
        }
    }

    #[test]
    fn lower_is_constructed() {
        let source = parse_source_expr("exp(x0) - log(x1)").unwrap();
        let eml = lower_to_eml(&source).unwrap();
        match eml {
            LoweredExpr::Eml(_, _) => {}
            _ => panic!("unexpected lowered tree shape: {eml:?}"),
        }
    }

    #[test]
    fn eval_reference_path() {
        let source = parse_source_expr("exp(x0) - log(x1)").unwrap();
        let vars = [Complex64::new(0.2, -0.1), Complex64::new(1.4, 0.3)];
        let val = eval_source_expr_complex(&source, &vars).unwrap();
        let ref_v = vars[0].exp() - vars[1].ln();
        assert!((val - ref_v).norm() <= 1e-12);
    }

    #[test]
    fn decimal_literal_becomes_rational() {
        let expr = parse_source_expr("0.125").unwrap();
        assert_eq!(expr, SourceExpr::Rational(1, 8));
    }

    #[test]
    fn parse_extended_function_family() {
        let expr = parse_source_expr("tanh(x0) + asin(x1) + gelu(x2)").unwrap();
        match expr {
            SourceExpr::Add(_, _) => {}
            _ => panic!("unexpected parse tree: {expr:?}"),
        }
    }

    #[test]
    fn lower_extended_functions_is_constructed() {
        let src = parse_source_expr("sigmoid(x0) + softplus(x1) + swish(x2)").unwrap();
        let lowered = lower_to_eml(&src).unwrap();
        match lowered {
            LoweredExpr::Eml(_, _) => {}
            _ => panic!("unexpected lowered tree: {lowered:?}"),
        }
    }

    #[test]
    fn eval_reference_for_ai_functions() {
        let src = parse_source_expr("sigmoid(x0) + softplus(x1) + relu(x2)").unwrap();
        let vars = [
            Complex64::new(0.3, 0.0),
            Complex64::new(-0.7, 0.0),
            Complex64::new(1.2, 0.0),
        ];
        let v = eval_source_expr_complex(&src, &vars).unwrap();
        assert!(v.re.is_finite() && v.im.is_finite());
    }

    #[test]
    fn parse_additional_ai_activations() {
        let src = parse_source_expr("elu(x0) + leaky_relu(x0) + softsign(x0) + mish(x0)").unwrap();
        match src {
            SourceExpr::Add(_, _) => {}
            _ => panic!("unexpected parse tree: {src:?}"),
        }
    }

    #[test]
    fn vector_templates_for_softmax_and_ce() {
        let logits = vec![
            SourceExpr::var(0),
            SourceExpr::Add(Box::new(SourceExpr::var(1)), Box::new(SourceExpr::Int(1))),
            SourceExpr::Sub(Box::new(SourceExpr::var(2)), Box::new(SourceExpr::Int(1))),
        ];
        let probs = softmax_template(&logits).unwrap();
        assert_eq!(probs.len(), logits.len());
        let ce = cross_entropy_template(&logits, 1).unwrap();
        let vars = [
            Complex64::new(0.1, 0.0),
            Complex64::new(0.5, 0.0),
            Complex64::new(-0.2, 0.0),
        ];
        let ce_val = eval_source_expr_complex(&ce, &vars).unwrap();
        assert!(ce_val.re.is_finite());
    }

    #[test]
    fn vector_templates_for_label_smoothing_and_focal() {
        let logits = vec![
            SourceExpr::var(0),
            SourceExpr::Add(Box::new(SourceExpr::var(1)), Box::new(SourceExpr::Int(1))),
            SourceExpr::Sub(Box::new(SourceExpr::var(2)), Box::new(SourceExpr::Int(1))),
        ];
        let ls = label_smoothing_cross_entropy_template(&logits, 1, SourceExpr::Rational(1, 10))
            .unwrap();
        let focal = focal_loss_template_with_alpha(
            &logits,
            1,
            SourceExpr::Int(2),
            SourceExpr::Rational(1, 4),
        )
        .unwrap();
        let vars = [
            Complex64::new(0.1, 0.0),
            Complex64::new(0.5, 0.0),
            Complex64::new(-0.2, 0.0),
        ];

        let ls_val = eval_source_expr_complex(&ls, &vars).unwrap().re;
        let focal_val = eval_source_expr_complex(&focal, &vars).unwrap().re;

        let z0 = 0.1_f64;
        let z1 = 1.5_f64;
        let z2 = -1.2_f64;
        let lse = (z0.exp() + z1.exp() + z2.exp()).ln();
        let mean = (z0 + z1 + z2) / 3.0;
        let expected_ls = lse - (0.9 * z1 + 0.1 * mean);
        assert!((ls_val - expected_ls).abs() <= 1e-12);

        let p_t = (z1 - lse).exp();
        let ce = lse - z1;
        let expected_focal = 0.25 * (1.0 - p_t).powf(2.0) * ce;
        assert!((focal_val - expected_focal).abs() <= 1e-12);
    }

    #[test]
    fn batch_label_smoothing_and_focal_mean_templates_match_manual_mean() {
        let batch_logits = vec![
            vec![SourceExpr::var(0), SourceExpr::var(1), SourceExpr::var(2)],
            vec![
                SourceExpr::var(3),
                SourceExpr::Add(Box::new(SourceExpr::var(4)), Box::new(SourceExpr::Int(1))),
                SourceExpr::var(5),
            ],
        ];
        let targets = vec![1usize, 2usize];
        let ls_losses = batch_label_smoothing_cross_entropy_template(
            &batch_logits,
            &targets,
            SourceExpr::Rational(1, 10),
        )
        .unwrap();
        let ls_mean = batch_label_smoothing_cross_entropy_mean_template(
            &batch_logits,
            &targets,
            SourceExpr::Rational(1, 10),
        )
        .unwrap();

        let focal_losses = batch_focal_loss_template_with_alpha(
            &batch_logits,
            &targets,
            SourceExpr::Int(2),
            SourceExpr::Rational(1, 4),
        )
        .unwrap();
        let focal_mean = batch_focal_loss_mean_template_with_alpha(
            &batch_logits,
            &targets,
            SourceExpr::Int(2),
            SourceExpr::Rational(1, 4),
        )
        .unwrap();

        let vars = [
            Complex64::new(0.2, 0.0),
            Complex64::new(0.7, 0.0),
            Complex64::new(-0.1, 0.0),
            Complex64::new(1.0, 0.0),
            Complex64::new(-0.2, 0.0),
            Complex64::new(0.3, 0.0),
        ];

        let ls0 = eval_source_expr_complex(&ls_losses[0], &vars).unwrap();
        let ls1 = eval_source_expr_complex(&ls_losses[1], &vars).unwrap();
        let ls_avg = eval_source_expr_complex(&ls_mean, &vars).unwrap();
        assert!((ls_avg - (ls0 + ls1) / Complex64::new(2.0, 0.0)).norm() <= 1e-12);

        let fl0 = eval_source_expr_complex(&focal_losses[0], &vars).unwrap();
        let fl1 = eval_source_expr_complex(&focal_losses[1], &vars).unwrap();
        let fl_avg = eval_source_expr_complex(&focal_mean, &vars).unwrap();
        assert!((fl_avg - (fl0 + fl1) / Complex64::new(2.0, 0.0)).norm() <= 1e-12);
    }

    #[test]
    fn symbolic_derivative_matches_finite_difference() {
        let expr = parse_source_expr("exp(x0) * log(x1 + 2)").unwrap();
        let d_dx0 = symbolic_derivative(&expr, 0);
        let d_dx1 = symbolic_derivative(&expr, 1);
        let vars = [Complex64::new(0.3, 0.0), Complex64::new(1.2, 0.0)];

        let analytic0 = eval_source_expr_complex(&d_dx0, &vars).unwrap().re;
        let numeric0 = finite_diff_real(&expr, &vars, 0, 1e-6);
        assert!((analytic0 - numeric0).abs() <= 1e-4);

        let analytic1 = eval_source_expr_complex(&d_dx1, &vars).unwrap().re;
        let numeric1 = finite_diff_real(&expr, &vars, 1, 1e-6);
        assert!((analytic1 - numeric1).abs() <= 1e-4);
    }

    #[test]
    fn symbolic_derivative_for_activation_surrogates_is_evaluable() {
        let expr = parse_source_expr("mish(x0) + elu(x0,0.5) + leaky_relu(x0,0.1)").unwrap();
        let deriv = symbolic_derivative(&expr, 0);
        let vars = [Complex64::new(0.25, 0.0)];
        let analytic = eval_source_expr_complex(&deriv, &vars).unwrap().re;
        let numeric = finite_diff_real(&expr, &vars, 0, 1e-6);
        assert!((analytic - numeric).abs() <= 5e-3);
    }

    #[test]
    fn simplify_source_expr_removes_identity_patterns() {
        let expr = parse_source_expr("((x0 * 1) + 0)^1").unwrap();
        let simplified = simplify_source_expr(&expr);
        assert_eq!(simplified, SourceExpr::var(0));
    }

    #[test]
    fn symbolic_derivative_simplifies_power_tree_size() {
        let expr = parse_source_expr("x0^8").unwrap();
        let deriv = symbolic_derivative(&expr, 0);
        let vars = [Complex64::new(2.0, 0.0)];
        let value = eval_source_expr_complex(&deriv, &vars).unwrap();
        assert!((value.re - 1024.0).abs() <= 1e-9);
        assert!(source_expr_node_count(&deriv) <= 12, "{deriv:?}");
    }

    #[test]
    fn batch_templates_are_shape_consistent_and_mean_matches() {
        let batch_logits = vec![
            vec![SourceExpr::var(0), SourceExpr::var(1), SourceExpr::var(2)],
            vec![SourceExpr::var(3), SourceExpr::var(4), SourceExpr::var(5)],
        ];
        let targets = vec![1usize, 2usize];

        let probs = batch_softmax_template(&batch_logits).unwrap();
        assert_eq!(probs.len(), 2);
        assert_eq!(probs[0].len(), 3);
        assert_eq!(probs[1].len(), 3);

        let losses = batch_cross_entropy_template(&batch_logits, &targets).unwrap();
        assert_eq!(losses.len(), 2);
        let mean_loss = batch_cross_entropy_mean_template(&batch_logits, &targets).unwrap();

        let vars = [
            Complex64::new(0.2, 0.0),
            Complex64::new(0.7, 0.0),
            Complex64::new(-0.1, 0.0),
            Complex64::new(1.0, 0.0),
            Complex64::new(-0.2, 0.0),
            Complex64::new(0.3, 0.0),
        ];
        let l0 = eval_source_expr_complex(&losses[0], &vars).unwrap();
        let l1 = eval_source_expr_complex(&losses[1], &vars).unwrap();
        let mean = eval_source_expr_complex(&mean_loss, &vars).unwrap();
        let expected = (l0 + l1) / Complex64::new(2.0, 0.0);
        assert!((mean - expected).norm() <= 1e-10);
    }

    #[test]
    fn delowered_source_matches_lowered_eval() {
        let source = parse_source_expr("sigmoid(x0) + softplus(x1) - log(x2 + 3)").unwrap();
        let lowered = lower_to_eml(&source).unwrap();
        let raised = delower_to_source(&lowered);
        let vars = [
            Complex64::new(0.2, 0.0),
            Complex64::new(-0.3, 0.0),
            Complex64::new(0.8, 0.0),
        ];
        let lowered_v = eval_lowered_expr_complex(&lowered, &vars).unwrap();
        let raised_v = eval_source_expr_complex(&raised, &vars).unwrap();
        assert!((lowered_v - raised_v).norm() <= 1e-12);
    }
}
