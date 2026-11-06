//! Test adapter imports production authorities without copying their implementations.
#[path = "../../src/doc/export/contract.rs"]
#[allow(dead_code)]
mod contract;
#[path = "../../src/doc/export/document.rs"]
#[allow(dead_code)]
mod document;
#[path = "../../src/doc/slug.rs"]
mod slug;

/// Preserve parser destinations verbatim; the isolated route applies the explicit URI fixture map.
/// Example: `/föö` remains raw data and maps to the same inert target as its encoded spelling.
fn target(url: &str) -> contract::Target {
    contract::Target::External {
        url: url.into(),
    }
}

pub(super) fn run(
    input: &std::path::Path,
    output: &std::path::Path,
) -> Result<(), Box<dyn std::error::Error>> {
    let cases: Vec<serde_json::Value> = serde_json::from_slice(&std::fs::read(input)?)?;
    let records = cases
        .iter()
        .map(|case| {
            let source = case["markdown"].as_str().ok_or("missing Markdown source")?;
            let mut upstream = String::new();
            pulldown_cmark::html::push_html(&mut upstream, pulldown_cmark::Parser::new(source));
            let mut effective = String::new();
        pulldown_cmark::html::push_html(&mut effective, pulldown_cmark::Parser::new_ext(source,
            pulldown_cmark::Options::ENABLE_TABLES | pulldown_cmark::Options::ENABLE_STRIKETHROUGH));
        let nodes = document::parse(
                source,
                0,
                true,
                &|url| Ok(Some(target(url))),
                &|info, code| {
                    Ok(contract::DocumentNode::CodeBlock {
                        source: contract::ResolvedText {
                            source: code.into(),
                            spans: Vec::new(),
                        },
                        language: info.split([' ', ',']).next().unwrap_or("").into(),
                        info: info.into(),
                    })
                },
            )?;
            Ok(serde_json::json!({"id":case["example"],"upstreamHtml":upstream,"effectiveHtml":effective,"nodes":nodes}))
        })
        .collect::<Result<Vec<_>, Box<dyn std::error::Error>>>()?;
    std::fs::write(output, serde_json::to_vec(&records)?)?;
    Ok(())
}
