//! Fixture-only driver for the exact production lexical coloring interpreter.
#[path = "../src/doc/coloring.rs"]
mod coloring;

fn main() {
    let mut args = std::env::args().skip(1);
    let input = args.next().expect("fixture path");
    let output = args.next().expect("output path");
    assert!(args.next().is_none());
    let cases: serde_json::Value = serde_json::from_slice(&std::fs::read(input).unwrap()).unwrap();
    let result: Vec<_> = cases.as_array().unwrap().iter().map(|case| {
        let source: String = case["segments"].as_array().unwrap().iter().map(|part| part[1].as_str().unwrap()).collect();
        let mut spans: Vec<serde_json::Value> = Vec::new();
        let lexical = coloring::scan(&source);
        for token in &lexical {
            if let Some(previous) = spans.last_mut() && previous["role"].as_str() == token.role {
                previous["end"] = token.end.into();
            } else { spans.push(serde_json::json!({"start":token.start,"end":token.end,"role":token.role})); }
        }
        serde_json::json!({"id":case["id"],"source":source,"spans":spans,"lexical":lexical.iter().map(|token|serde_json::json!({"start":token.start,"end":token.end,"role":token.role,"identifier":token.kind==coloring::Kind::Identifier})).collect::<Vec<_>>()})
    }).collect();
    std::fs::write(output, serde_json::to_vec_pretty(&result).unwrap()).unwrap();
}
