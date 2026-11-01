//! A program's internal functions and globals never share a symbol name.
//!
//! Both are named `<name>.<index>` once, and indices of functions and globals count apart, so a
//! function and a global with the same source name could land on the same text. Which of the two
//! then kept the name and which LLVM renamed (`next_id.245.1`) depended on the order each code
//! module happened to declare them in, so the modules of a split build disagreed about the
//! symbol: two objects defined one global, and another referred to a name nothing defined (an
//! ELF linker reports both).
use test_support::{TempDir, backend as support};

const SOURCE: &str = r"
next_id: s64;

bump :: () -> s64 {
    next_id += 1;
    return next_id;
}

#program_export 'answer' answer :: () -> s64 {
    local :: () -> s64 { return bump(); }
    next_id :: () -> s64 { return local() + 1; }
    return next_id();
}
";

#[test]
fn a_global_and_a_function_of_one_name_get_different_symbols() {
    let variant = support::VARIANTS[0];
    let compiler = variant.compile(&SOURCE.replace('\'', "\""));
    let dir = TempDir::new("symbol-names");
    let ir = dir.join("module.ll");
    variant.emit(&compiler, &dir.join("module.o"), Some(&ir));
    let text = std::fs::read_to_string(&ir).unwrap();
    let mut globals = Vec::new();
    let mut functions = Vec::new();
    for line in text.lines() {
        let Some(rest) = line.strip_prefix('@') else {
            if let Some(at) = line
                .strip_prefix("define ")
                .and_then(|l| l.split('@').nth(1))
            {
                functions.push(at.split('(').next().unwrap().trim_matches('"').to_string());
            }
            continue;
        };
        globals.push(
            rest.split(" = ")
                .next()
                .unwrap()
                .trim_matches('"')
                .to_string(),
        );
    }
    let ours = |names: &[String]| -> Vec<String> {
        names
            .iter()
            .filter(|n| n.starts_with("next_id."))
            .cloned()
            .collect()
    };
    let (globals, functions) = (ours(&globals), ours(&functions));
    assert!(!globals.is_empty() && !functions.is_empty(), "{text}");
    for g in &globals {
        assert!(!functions.contains(g), "{g} names a global and a function");
        assert!(g.rsplit('.').next().unwrap().starts_with('g'), "{g}");
    }
}
