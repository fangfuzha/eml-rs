use std::process::Command;

use eml_rs::core::{eml_complex, eml_real};
use num_complex::Complex64;
use serde_json::Value;

fn run_python_backend(
    backend: &str,
    x: Complex64,
    y: Complex64,
) -> Option<Result<Complex64, String>> {
    let script = r#"
import json, sys
backend = sys.argv[1]
x = complex(float(sys.argv[2]), float(sys.argv[3]))
y = complex(float(sys.argv[4]), float(sys.argv[5]))

if backend == "mpmath":
    import mpmath as mp
    xv = mp.mpc(x.real, x.imag)
    yv = mp.mpc(y.real, y.imag)
    v = mp.e ** xv - mp.log(yv)
    out = {"re": float(mp.re(v)), "im": float(mp.im(v))}
elif backend == "torch":
    import torch
    xv = torch.tensor(complex(x.real, x.imag), dtype=torch.complex128)
    yv = torch.tensor(complex(y.real, y.imag), dtype=torch.complex128)
    v = torch.exp(xv) - torch.log(yv)
    out = {"re": float(torch.real(v)), "im": float(torch.imag(v))}
else:
    raise RuntimeError(f"unknown backend: {backend}")

print(json.dumps(out))
"#;

    let args = [
        "-c",
        script,
        backend,
        &x.re.to_string(),
        &x.im.to_string(),
        &y.re.to_string(),
        &y.im.to_string(),
    ];

    let try_python = Command::new("python").args(args).output();
    let output = match try_python {
        Ok(o) => o,
        Err(_) => match Command::new("py").args(["-3"]).args(args).output() {
            Ok(o) => o,
            Err(_) => return None,
        },
    };

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();
        return Some(Err(stderr));
    }

    let text = String::from_utf8_lossy(&output.stdout);
    let value: Value = serde_json::from_str(text.trim()).ok()?;
    let re = value.get("re")?.as_f64()?;
    let im = value.get("im")?.as_f64()?;
    Some(Ok(Complex64::new(re, im)))
}

#[test]
fn cross_backend_libm_real_path() {
    let samples = [(0.3, 1.1), (-0.7, 2.5), (3.2, 0.4), (0.0, 10.0)];
    for (x, y) in samples {
        let rust = eml_real(x, y).unwrap();
        let libm_ref = libm::exp(x) - libm::log(y);
        assert!((rust - libm_ref).abs() <= 1e-12, "x={x}, y={y}");
    }
}

#[test]
fn cross_backend_mpmath_complex128_optional() {
    let x = Complex64::new(0.3, 0.2);
    let y = Complex64::new(1.4, -0.5);
    let rust = eml_complex(x, y).unwrap();

    let Some(maybe) = run_python_backend("mpmath", x, y) else {
        eprintln!("skip: python runtime not found");
        return;
    };
    let py = match maybe {
        Ok(v) => v,
        Err(err) => {
            eprintln!("skip: mpmath backend unavailable: {err}");
            return;
        }
    };
    assert!((rust - py).norm() <= 1e-10, "rust={rust:?}, mpmath={py:?}");
}

#[test]
fn cross_backend_torch_complex128_optional() {
    let x = Complex64::new(0.3, 0.2);
    let y = Complex64::new(1.4, -0.5);
    let rust = eml_complex(x, y).unwrap();

    let Some(maybe) = run_python_backend("torch", x, y) else {
        eprintln!("skip: python runtime not found");
        return;
    };
    let py = match maybe {
        Ok(v) => v,
        Err(err) => {
            eprintln!("skip: torch backend unavailable: {err}");
            return;
        }
    };
    assert!((rust - py).norm() <= 1e-10, "rust={rust:?}, torch={py:?}");
}
