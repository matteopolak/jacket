//! Batch the existing formatter over verified private source, then map compiler identities by
//! exact non-comment token order. Astro receives formatted text and never formats Jai syntax.
use super::{model, site::Site};
use jaic::lexer::{Tok, Token, lex};
use jaic::source::FileId;
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::Arc;

pub(super) const MAX_WIDTH: u32 = 88;
pub(super) const INDENT_WIDTH: u32 = 4;

type DefinitionKey = (PathBuf, u32, u32, Vec<(PathBuf, u32, u32, bool)>);

fn digest(bytes: &[u8]) -> String {
    Sha256::digest(bytes)
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

fn key(definition: &model::Definition) -> DefinitionKey {
    (
        definition.native_file.clone(),
        definition.start,
        definition.end,
        definition
            .guards
            .iter()
            .map(|guard| {
                (
                    guard.native_file.clone(),
                    guard.start,
                    guard.end,
                    guard.taken,
                )
            })
            .collect(),
    )
}

fn code_tokens(source: &str) -> Result<Vec<Token>, String> {
    lex(FileId(0), source)
        .map(|tokens| {
            tokens
                .into_iter()
                .filter(|token| !matches!(token.tok, Tok::Eof))
                .collect()
        })
        .map_err(|_| "documentation formatter input or output does not lex".into())
}

/// Corresponding token boundaries map `field : Type` onto formatter-produced `field: Type`.
#[derive(Debug)]
pub(super) struct TokenMap {
    source: Vec<(usize, usize)>,
    formatted: Vec<(usize, usize)>,
}

impl TokenMap {
    pub(super) fn pair(source: &str, formatted: &str) -> Result<Self, String> {
        let before = code_tokens(source)?;
        let after = code_tokens(formatted)?;
        if before.len() != after.len() {
            return Err("documentation formatter changed the code token count".into());
        }
        for (before, after) in before.iter().zip(&after) {
            let a = &source[before.span.start as usize..before.span.end as usize];
            let b = &formatted[after.span.start as usize..after.span.end as usize];
            if a != b {
                return Err(
                    "documentation formatter changed a code token or literal spelling".into(),
                );
            }
        }
        Ok(Self {
            source: before
                .iter()
                .map(|token| (token.span.start as usize, token.span.end as usize))
                .collect(),
            formatted: after
                .iter()
                .map(|token| (token.span.start as usize, token.span.end as usize))
                .collect(),
        })
    }

    /// A merged compiler path may cover several lexical tokens, e.g. `Pool . Pool`.
    pub(super) fn range(&self, start: usize, end: usize) -> Result<(usize, usize), String> {
        let first = self
            .source
            .binary_search_by_key(&start, |range| range.0)
            .map_err(|_| "compiler definition start is not a code token boundary")?;
        let last = self
            .source
            .binary_search_by_key(&end, |range| range.1)
            .map_err(|_| "compiler definition end is not a code token boundary")?;
        if last < first {
            return Err("compiler definition token range is reversed".into());
        }
        Ok((self.formatted[first].0, self.formatted[last].1))
    }
}

pub(super) struct Prepared {
    pub source: Arc<str>,
    body_start: usize,
    map: TokenMap,
}

impl Prepared {
    pub(super) fn range(
        &self,
        definition: &model::Definition,
        start: u32,
        end: u32,
    ) -> Result<(usize, usize), String> {
        if start < definition.start || end > definition.end || start >= end {
            return Err("compiler definition identity escapes its source span".into());
        }
        self.map.range(
            self.body_start + (start - definition.start) as usize,
            self.body_start + (end - definition.start) as usize,
        )
    }
}

/// Public formatter identity is recorded without exposing its native installation path.
pub(super) struct Identity {
    pub version: String,
    pub sha256: String,
}

pub(super) struct Batch {
    pub identity: Option<Identity>,
    keys: BTreeMap<DefinitionKey, String>,
    programs: BTreeMap<String, Prepared>,
}

impl Batch {
    pub(super) fn get(&self, definition: &model::Definition) -> Result<&Prepared, String> {
        self.keys
            .get(&key(definition))
            .and_then(|key| self.programs.get(key))
            .ok_or_else(|| "definition was not included in the formatter batch".into())
    }

    /// Format all unique declaration programs in one owned directory and one tool invocation.
    /// For example, 5,000 API routes still start only one `jaifmt --declarations` process.
    pub(super) fn prepare(site: &Site, work: &Path) -> Result<Self, String> {
        let mut cache = BTreeMap::new();
        let mut keys = BTreeMap::new();
        let mut inputs = BTreeMap::<String, (String, usize)>::new();
        for definition in site
            .modules
            .iter()
            .flat_map(|module| module.items.iter().chain(&module.private))
            .flat_map(|item| &item.definitions)
        {
            let (program, body_start) = program(definition, &mut cache)?;
            let hash = digest(program.as_bytes());
            keys.insert(key(definition), hash.clone());
            inputs.entry(hash).or_insert((program, body_start));
        }
        if inputs.is_empty() {
            return Ok(Self {
                identity: None,
                keys,
                programs: BTreeMap::new(),
            });
        }
        let formatter = crate::tools::locate(crate::tools::JAIFMT)?;
        let tool_bytes = std::fs::read(&formatter)
            .map_err(|error| format!("read documentation formatter: {error}"))?;
        let sha256 = digest(&tool_bytes);
        let version = Command::new(&formatter)
            .arg("--version")
            .output()
            .map_err(|error| format!("start documentation formatter: {error}"))?;
        if !version.status.success() {
            return Err("documentation formatter did not report its version".into());
        }
        let version = String::from_utf8(version.stdout)
            .map_err(|_| "documentation formatter version is not UTF-8")?
            .trim()
            .to_owned();
        if version.is_empty() || version.len() > 256 {
            return Err("documentation formatter reported an invalid version".into());
        }
        std::fs::create_dir(work)
            .map_err(|error| format!("create owned documentation formatter directory: {error}"))?;
        let owned = Directory(work.to_path_buf());
        let mut files = BTreeMap::new();
        for (index, (hash, (program, _))) in inputs.iter().enumerate() {
            let file = work.join(format!("definition-{index:06}.jai"));
            std::fs::write(&file, program)
                .map_err(|error| format!("stage documentation formatter input: {error}"))?;
            files.insert(hash.clone(), file);
        }
        let output = Command::new(&formatter)
            .args([
                "--declarations",
                "--no-config",
                "--set",
                "indent_width=4",
                "--set",
                "max_width=88",
            ])
            .arg(work)
            .output()
            .map_err(|error| format!("run documentation formatter: {error}"))?;
        if !output.status.success() {
            let details = format!(
                "{}{}",
                String::from_utf8_lossy(&output.stdout),
                String::from_utf8_lossy(&output.stderr)
            );
            let failed = files
                .iter()
                .find(|(_, file)| details.contains(&format!("{}: error:", file.display())));
            let context = failed
                .map(|(hash, _)| {
                    let source = &inputs[hash].0;
                    let excerpt = source
                        .char_indices()
                        .nth(8192)
                        .map_or(source.as_str(), |(end, _)| &source[..end]);
                    format!("\nFirst failing declaration input (sha256 {hash}):\n{excerpt}")
                })
                .unwrap_or_default();
            return Err(format!(
                "documentation declaration formatting failed: {details}{context}"
            ));
        }
        if digest(&std::fs::read(&formatter).map_err(|error| error.to_string())?) != sha256 {
            return Err("documentation formatter changed while the export was running".into());
        }
        let mut programs = BTreeMap::new();
        for (hash, (input, body_start)) in inputs {
            let formatted = std::fs::read_to_string(&files[&hash])
                .map_err(|error| format!("read documentation formatter result: {error}"))?;
            let map = TokenMap::pair(&input, &formatted)?;
            programs.insert(
                hash,
                Prepared {
                    source: Arc::from(formatted),
                    body_start,
                    map,
                },
            );
        }
        drop(owned);
        Ok(Self {
            identity: Some(Identity {
                version,
                sha256,
            }),
            keys,
            programs,
        })
    }
}

struct Directory(PathBuf);

impl Drop for Directory {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

type SourceCache = BTreeMap<PathBuf, (String, Arc<str>)>;

fn slice(
    cache: &mut SourceCache,
    file: &Path,
    expected: &str,
    start: u32,
    end: u32,
) -> Result<String, String> {
    if !cache.contains_key(file) {
        let bytes = std::fs::read(file)
            .map_err(|error| format!("read compiler-pinned declaration source: {error}"))?;
        let hash = digest(&bytes);
        let source = String::from_utf8(bytes)
            .map_err(|_| "compiler-pinned declaration source is not UTF-8")?;
        cache.insert(file.to_path_buf(), (hash, Arc::from(source)));
    }
    let (actual, source) = &cache[file];
    if actual != expected {
        return Err("declaration source changed after the compiler read it; export again".into());
    }
    source
        .get(start as usize..end as usize)
        .map(str::to_owned)
        .ok_or_else(|| "compiler definition source span is outside a UTF-8 source file".into())
}

fn program(
    definition: &model::Definition,
    cache: &mut SourceCache,
) -> Result<(String, usize), String> {
    let mut program = String::new();
    for guard in &definition.guards {
        let condition = slice(
            cache,
            &guard.native_file,
            &guard.input_sha256,
            guard.start,
            guard.end,
        )?;
        program.push_str("#if ");
        if !guard.taken {
            program.push_str("!(");
        }
        program.push_str(&condition);
        if !guard.taken {
            program.push(')');
        }
        program.push_str(" {\n");
    }
    let body_start = program.len();
    program.push_str(&slice(
        cache,
        &definition.native_file,
        &definition.input_sha256,
        definition.start,
        definition.end,
    )?);
    if !program.ends_with('\n') {
        program.push('\n');
    }
    for _ in &definition.guards {
        program.push_str("}\n");
    }
    Ok((program, body_start))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn token_ordinals_preserve_field_identity_qualified_paths_unicode_and_literals() {
        let input = "Record :: struct { état, other : *Other . Type; text := \"a  // b\"; }";
        let output =
            "Record :: struct {\n    état, other: *Other.Type;\n    text := \"a  // b\";\n}\n";
        let map = TokenMap::pair(input, output).unwrap();
        for (source, written) in [
            ("état", "état"),
            ("Other . Type", "Other.Type"),
            ("\"a  // b\"", "\"a  // b\""),
        ] {
            let start = input.find(source).unwrap();
            let (from, to) = map.range(start, start + source.len()).unwrap();
            assert_eq!(&output[from..to], written);
        }
        for changed in [
            output.replace("a  // b", "a // b"),
            output.replace("état", "state"),
            output.replace(", other", " other"),
        ] {
            assert!(TokenMap::pair(input, &changed).is_err());
        }
        assert!(map.range(1, 5).is_err());
    }

    #[test]
    fn verified_private_source_rejects_changed_bytes_and_non_utf8_boundaries() {
        let temp = test_support::TempDir::new("doc-format-source");
        temp.write("source.jai", "é: int;\n");
        let file = temp.join("source.jai");
        let hash = digest("é: int;\n".as_bytes());
        let mut cache = SourceCache::new();
        assert_eq!(slice(&mut cache, &file, &hash, 0, 2).unwrap(), "é");
        assert!(slice(&mut cache, &file, &hash, 1, 2).is_err());
        assert!(slice(&mut cache, &file, &"0".repeat(64), 0, 2).is_err());
    }
}
