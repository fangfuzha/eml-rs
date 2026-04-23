use std::sync::{Arc, Mutex};

use eml_rs::api::{compile, BuiltinBackend, PipelineBuilder, PipelineOptions};
use eml_rs::core::EvalPolicy;
use eml_rs::ir::Expr;
use eml_rs::lowering::SourceExpr;
use eml_rs::plugin::{
    ExecutionBackend, ExprPass, PipelineEvent, PipelineObserver, PipelineStage, SourcePass,
};
use eml_rs::{EmlErrorCode, EmlResult};
use num_complex::Complex64;

struct LogExpIdentityPass;

impl SourcePass for LogExpIdentityPass {
    fn name(&self) -> &'static str {
        "log_exp_identity"
    }

    fn run(&self, expr: &SourceExpr) -> EmlResult<SourceExpr> {
        Ok(SourceExpr::Log(Box::new(SourceExpr::Exp(Box::new(
            expr.clone(),
        )))))
    }
}

struct DuplicateExprPass;

impl ExprPass for DuplicateExprPass {
    fn name(&self) -> &'static str {
        "duplicate_expr"
    }

    fn run(&self, expr: &Expr) -> EmlResult<Expr> {
        Ok(Expr::eml(expr.clone(), expr.clone()))
    }
}

#[derive(Clone)]
struct EventCollector(Arc<Mutex<Vec<PipelineEvent>>>);

impl PipelineObserver for EventCollector {
    fn on_event(&self, event: &PipelineEvent) {
        self.0.lock().unwrap().push(event.clone());
    }
}

struct TreeBackend;

impl ExecutionBackend for TreeBackend {
    fn name(&self) -> &'static str {
        "tree"
    }

    fn eval_complex(
        &self,
        expr: &Expr,
        vars: &[Complex64],
        policy: &EvalPolicy,
    ) -> EmlResult<Complex64> {
        expr.eval_complex_with_policy(vars, policy)
    }
}

#[test]
fn compile_default_pipeline_supports_all_builtin_backends() {
    let compiled = compile("exp(x0) - log(x1)").unwrap();
    let vars_complex = [Complex64::new(0.4, 0.1), Complex64::new(1.6, -0.2)];
    let tree = compiled
        .eval_complex(BuiltinBackend::Tree, &vars_complex)
        .unwrap();
    let rpn = compiled
        .eval_complex(BuiltinBackend::Rpn, &vars_complex)
        .unwrap();
    let bytecode = compiled
        .eval_complex(BuiltinBackend::Bytecode, &vars_complex)
        .unwrap();
    assert!((tree - rpn).norm() <= 1e-12);
    assert!((tree - bytecode).norm() <= 1e-12);

    let vars_real = [0.4, 1.6];
    let real = compiled
        .eval_real(BuiltinBackend::Bytecode, &vars_real)
        .unwrap();
    assert!(real.is_finite());
}

#[test]
fn pipeline_builder_runs_custom_passes_and_observers() {
    let events = Arc::new(Mutex::new(Vec::<PipelineEvent>::new()));
    let compiled = PipelineBuilder::new()
        .with_source_pass(LogExpIdentityPass)
        .with_expr_pass(DuplicateExprPass)
        .with_observer(EventCollector(events.clone()))
        .compile_str("x0")
        .unwrap();

    let report = compiled.report();
    assert!(report.input_source_nodes >= 1);
    assert!(report.optimized_source_nodes >= report.input_source_nodes);
    assert!(report.expr_stats.nodes >= 3);
    assert!(report.bytecode_instructions.is_some());

    let seen = events.lock().unwrap();
    assert!(seen.iter().any(|e| e.stage == PipelineStage::Parsed));
    assert!(seen.iter().any(|e| e.stage == PipelineStage::SourcePass));
    assert!(seen.iter().any(|e| e.stage == PipelineStage::ExprPass));
    assert!(seen
        .iter()
        .any(|e| e.stage == PipelineStage::BytecodeCompiled));
}

#[test]
fn pipeline_can_verify_against_reference_and_custom_backend() {
    let compiled = compile("exp(x0)").unwrap();
    let backend = TreeBackend;
    let vars = [Complex64::new(0.3, 0.0)];
    let backend_value = compiled.eval_complex_with_backend(&backend, &vars).unwrap();
    let tree_value = compiled.eval_complex(BuiltinBackend::Tree, &vars).unwrap();
    assert!((backend_value - tree_value).norm() <= 1e-12);

    let samples = vec![vec![0.1], vec![0.3], vec![1.0], vec![2.0]];
    let report = compiled.verify_against_real_ref(&samples, 1e-9, |xs| xs[0].exp());
    assert!(report.all_passed(), "{report:?}");
}

#[test]
fn bytecode_backend_requires_precompilation() {
    let compiled = PipelineBuilder::new()
        .with_options(PipelineOptions {
            compile_bytecode: false,
            ..PipelineOptions::default()
        })
        .compile_str("x0")
        .unwrap();

    let err = compiled
        .eval_complex(BuiltinBackend::Bytecode, &[Complex64::new(0.2, 0.0)])
        .unwrap_err();
    assert_eq!(err.code(), EmlErrorCode::Unsupported);
}
