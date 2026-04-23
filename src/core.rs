use num_complex::Complex64;

use crate::EmlError;

pub fn is_finite_complex(v: Complex64) -> bool {
    v.re.is_finite() && v.im.is_finite()
}

pub fn eml_complex(x: Complex64, y: Complex64) -> Result<Complex64, EmlError> {
    if !is_finite_complex(x) {
        return Err(EmlError::NonFiniteInput("x is not finite"));
    }
    if !is_finite_complex(y) {
        return Err(EmlError::NonFiniteInput("y is not finite"));
    }
    if y == Complex64::new(0.0, 0.0) {
        return Err(EmlError::Domain("log(0) is undefined"));
    }

    let out = x.exp() - y.ln();
    if !is_finite_complex(out) {
        return Err(EmlError::NonFiniteOutput(
            "eml_complex produced non-finite value",
        ));
    }
    Ok(out)
}

pub fn eml_real(x: f64, y: f64) -> Result<f64, EmlError> {
    if !x.is_finite() {
        return Err(EmlError::NonFiniteInput("x is not finite"));
    }
    if !y.is_finite() {
        return Err(EmlError::NonFiniteInput("y is not finite"));
    }
    if y <= 0.0 {
        return Err(EmlError::Domain("real log(y) requires y > 0"));
    }

    let out = x.exp() - y.ln();
    if !out.is_finite() {
        return Err(EmlError::NonFiniteOutput(
            "eml_real produced non-finite value",
        ));
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn eml_real_matches_definition() {
        let x = 0.7;
        let y = 2.5;
        let got = eml_real(x, y).unwrap();
        let expected = x.exp() - y.ln();
        assert!((got - expected).abs() <= 1e-14);
    }

    #[test]
    fn eml_real_rejects_non_positive_y() {
        let err = eml_real(1.0, 0.0).unwrap_err();
        assert!(matches!(err, EmlError::Domain(_)));
    }
}
