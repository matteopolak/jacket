//! Generate the documentation interchange schema from its Rust authority.
//! Example: `cargo run -p jacket --example doc_site_schema -- website/docs/schema/doc-site.schema.json`.
#[path = "../src/doc/export/contract.rs"]
#[allow(dead_code)]
mod contract;

fn main() {
    let schema = schemars::schema_for!(contract::Payload);
    let mut bytes = serde_json::to_vec_pretty(&schema).expect("serialize schema");
    bytes.push(b'\n');
    let Some(path) = std::env::args_os().nth(1) else {
        eprintln!("usage: doc_site_schema OUTPUT.json");
        std::process::exit(2);
    };
    std::fs::write(path, bytes).expect("write schema");
}
