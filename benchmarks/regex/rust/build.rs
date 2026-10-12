// Builds one fully materialised dense DFA per case in ../cases.tsv and writes it to OUT_DIR as
// `<id>.dfa` (native-endian, ready for zero-copy `DFA::from_bytes`). A case whose pattern a dense
// DFA cannot express (Unicode \b, capture groups) gets an empty file; main.rs reports it as n/a
// with the build error from `<id>.err`.
use regex_automata::{
    dfa::dense,
    util::syntax,
    MatchKind,
};
use std::{env, fs, path::PathBuf};

fn main() {
    let manifest = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    let cases = manifest.join("../cases.tsv");
    println!("cargo:rerun-if-changed={}", cases.display());
    let out = PathBuf::from(env::var("OUT_DIR").unwrap());
    for line in fs::read_to_string(&cases).unwrap().lines() {
        if line.starts_with('#') || line.trim().is_empty() {
            continue;
        }
        let mut f = line.split('\t');
        let id = f.next().unwrap();
        let pattern = f.next().unwrap();
        let built = dense::Builder::new()
            .configure(
                dense::Config::new()
                    .match_kind(MatchKind::LeftmostFirst)
                    .start_kind(regex_automata::dfa::StartKind::Unanchored),
            )
            .syntax(syntax::Config::new())
            .build(pattern);
        match built {
            Ok(dfa) => {
                let (bytes, pad) = dfa.to_bytes_native_endian();
                fs::write(out.join(format!("{id}.dfa")), &bytes[pad..]).unwrap();
                fs::write(out.join(format!("{id}.err")), "").unwrap();
            }
            Err(e) => {
                fs::write(out.join(format!("{id}.dfa")), b"").unwrap();
                fs::write(out.join(format!("{id}.err")), e.to_string()).unwrap();
            }
        }
    }
}
