use std::fs;

#[test]
fn api_stability_document_defines_public_tiers() {
    let doc = fs::read_to_string("docs/api-stability.md")
        .expect("docs/api-stability.md should document API stability tiers");
    for required in [
        "Stable API",
        "Experimental API",
        "Internal API",
        "compile()",
        "PipelineBuilder",
        "Deprecated API",
    ] {
        assert!(
            doc.contains(required),
            "api stability document should mention {required}"
        );
    }
}

#[test]
fn deprecated_flow_has_a_code_level_example() {
    let api_rs = fs::read_to_string("src/api.rs").expect("src/api.rs should be readable");
    assert!(api_rs.contains("#[deprecated("));
    assert!(api_rs.contains("pub fn compile_expression"));
}

#[test]
fn ci_lint_runs_rustdoc_gate() {
    let ci =
        fs::read_to_string(".github/workflows/ci.yml").expect("ci workflow should be readable");
    assert!(ci.contains("cargo doc --workspace --no-deps"));
}
