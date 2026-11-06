//! Execute the production Markdown-to-data parser against the pinned conformance inventory.
//! Example: `cargo run -p jacket --example doc_commonmark -- spec.json producer.json`.
#[path = "doc_commonmark/mod.rs"]
mod doc;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = std::env::args_os().skip(1).collect::<Vec<_>>();
    if args.len() != 2 {
        return Err("usage: doc_commonmark CASES.json OUTPUT.json".into());
    }
    doc::run(
        std::path::Path::new(&args[0]),
        std::path::Path::new(&args[1]),
    )
}
