use eml_rs::api::{BuiltinBackend, PipelineBuilder};
use eml_rs::plugin::{PipelineEvent, PipelineObserver};
use std::sync::{Arc, Mutex};

#[derive(Clone)]
struct PrintObserver(Arc<Mutex<Vec<PipelineEvent>>>);

impl PipelineObserver for PrintObserver {
    fn on_event(&self, event: &PipelineEvent) {
        self.0.lock().unwrap().push(event.clone());
    }
}

fn main() {
    let events = Arc::new(Mutex::new(Vec::<PipelineEvent>::new()));
    let compiled = PipelineBuilder::new()
        .with_observer(PrintObserver(events.clone()))
        .compile_str("sigmoid(x0) + softplus(x0)")
        .expect("pipeline compile");

    let value = compiled
        .eval_real(BuiltinBackend::Bytecode, &[0.35])
        .expect("bytecode eval");
    println!("value={value:.8}");
    println!("expr_nodes={}", compiled.report().expr_stats.nodes);
    println!("events={}", events.lock().unwrap().len());
}
