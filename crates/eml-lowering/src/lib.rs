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
    use super::*;

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
}
