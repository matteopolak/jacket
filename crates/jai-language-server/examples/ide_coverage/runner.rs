use super::report::{self, Counts, Failure, File, Gap, Manifest, Missing, Report, Target};
use super::{KEYWORDS, Role, environment, position, role, using_modifier};
use jai_language_server::{DiagnosticCode, DiagnosticSeverity, DocumentUri, Limits, Session};
use jaic::fxhash::HashSet;
use jaic::{
    lexer::{P, Tok},
    sema::{TargetCpu, TargetOs},
};
use std::io::Write;
use std::path::{Path, PathBuf};

fn checkpoint(path: Option<&Path>, value: serde_json::Value) -> Result<(), String> {
    if let Some(path) = path {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
        }
        let mut file = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(path)
            .map_err(|e| e.to_string())?;
        serde_json::to_writer(&mut file, &value).map_err(|e| e.to_string())?;
        file.write_all(b"\n").map_err(|e| e.to_string())?;
        file.sync_data().map_err(|e| e.to_string())?;
    }
    Ok(())
}

fn checkpoint_file(path: Option<&Path>, file: &File) -> Result<(), String> {
    if path.is_none() {
        return Ok(());
    }
    let mut file = serde_json::to_value(file).map_err(|e| e.to_string())?;
    if let Some(object) = file.as_object_mut() {
        object.remove("path");
    }
    for failure in file["failures"].as_array_mut().into_iter().flatten() {
        if let Some(object) = failure.as_object_mut() {
            object.remove("diagnostic");
        }
    }
    for gap in file["gaps"].as_array_mut().into_iter().flatten() {
        if let Some(object) = gap.as_object_mut() {
            object.remove("name");
            object.insert("definitions".into(), serde_json::json!([]));
        }
    }
    checkpoint(
        path,
        serde_json::json!({"kind":"file","file":file,"redacted":true}),
    )
}

fn uri(path: &Path) -> Result<DocumentUri, String> {
    let encoded = path
        .to_string_lossy()
        .replace('%', "%25")
        .replace('#', "%23")
        .replace('?', "%3F")
        .replace(' ', "%20");
    DocumentUri::parse(&format!("file://{encoded}")).map_err(|e| format!("{e:?}"))
}

fn walk(path: &Path, seen: &mut HashSet<PathBuf>, files: &mut Vec<Result<PathBuf, String>>) {
    let canonical = match path.canonicalize() {
        Ok(p) => p,
        Err(e) => {
            files.push(Err(e.to_string()));
            return;
        }
    };
    if !seen.insert(canonical.clone()) {
        return;
    }
    if canonical.is_file() {
        if canonical.extension().is_some_and(|e| e == "jai") {
            files.push(Ok(canonical));
        }
        return;
    }
    let entries = match std::fs::read_dir(&canonical) {
        Ok(entries) => entries,
        Err(e) => {
            files.push(Err(e.to_string()));
            return;
        }
    };
    let mut paths = Vec::new();
    for entry in entries {
        match entry {
            Ok(entry) => paths.push(entry.path()),
            Err(e) => files.push(Err(e.to_string())),
        }
    }
    paths.sort();
    for path in paths {
        walk(&path, seen, files);
    }
}

fn failure(kind: &str, diagnostic: String) -> Failure {
    Failure {
        kind: kind.into(),
        line: None,
        character: None,
        diagnostic: Some(diagnostic),
    }
}

/// The command line.
struct Args {
    json: bool,
    strict: bool,
    redacted: bool,
    runtime: bool,
    case_id: Option<String>,
    stdlib_id: String,
    stdlib: PathBuf,
    manifest: Manifest,
    os: Option<TargetOs>,
    cpu: Option<TargetCpu>,
    dirs: Vec<PathBuf>,
    progress: Option<PathBuf>,
    inputs: Option<Vec<String>>,
    selected: Option<HashSet<String>>,
}

fn parse_args() -> Result<Args, String> {
    let mut parsed = Args {
        json: false,
        strict: false,
        redacted: false,
        runtime: true,
        case_id: None,
        stdlib_id: "unverified-working-tree".to_string(),
        stdlib: PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../stdlib"),
        manifest: Manifest::default(),
        os: None,
        cpu: None,
        dirs: Vec::new(),
        progress: None,
        inputs: None,
        selected: None,
    };
    let mut args = std::env::args().skip(1);
    while let Some(arg) = args.next() {
        let next = |args: &mut std::iter::Skip<std::env::Args>| {
            args.next().ok_or_else(|| format!("{arg} requires a value"))
        };
        match arg.as_str() {
            "--json" => parsed.json = true,
            "--strict" => parsed.strict = true,
            "--redacted" => parsed.redacted = true,
            "--no-runtime" => parsed.runtime = false,
            "--case-id" => parsed.case_id = Some(next(&mut args)?),
            "--progress" => parsed.progress = Some(PathBuf::from(next(&mut args)?)),
            "--inputs" => {
                let bytes = std::fs::read(next(&mut args)?).map_err(|e| e.to_string())?;
                parsed.inputs = Some(serde_json::from_slice(&bytes).map_err(|e| e.to_string())?);
            }
            "--only-inputs" => {
                parsed.selected = Some(next(&mut args)?.split(',').map(str::to_owned).collect());
            }
            "--stdlib-id" => parsed.stdlib_id = next(&mut args)?,
            "--stdlib" => parsed.stdlib = PathBuf::from(next(&mut args)?),
            "--review" => {
                let bytes = std::fs::read(next(&mut args)?).map_err(|e| e.to_string())?;
                parsed.manifest = serde_json::from_slice(&bytes).map_err(|e| format!("review: {e}"))?;
            }
            "--os" => {
                parsed.os = Some(jaic::target::parse_os(&next(&mut args)?).ok_or("invalid --os")?);
            }
            "--cpu" => {
                parsed.cpu = Some(jaic::target::parse_cpu(&next(&mut args)?).ok_or("invalid --cpu")?);
            }
            // Replaced by structured file/gap/failure records.
            "--list" | "--errors" | "--stats" => {}
            "--help" => return Err("ide_coverage [--json] [--strict] [--redacted] [--review FILE] [--case-id ID] [--progress NEW_JSONL] [--inputs RELATIVE_PATHS_JSON] [--only-inputs ID,ID] [--stdlib DIR] [--stdlib-id ID] [--no-runtime] [--os OS] [--cpu CPU] DIR...".into()),
            _ if arg.starts_with('-') => return Err(format!("unknown option {arg}")),
            _ => parsed.dirs.push(PathBuf::from(arg)),
        }
    }
    if parsed.dirs.is_empty() {
        return Err("at least one explicit input directory/file is required".into());
    }
    parsed.manifest.validate()?;
    if parsed.os == Some(TargetOs::Wasm) && parsed.cpu.is_none() {
        parsed.cpu = Some(TargetCpu::Wasm);
    }
    if (parsed.os == Some(TargetOs::Wasm)) != (parsed.cpu == Some(TargetCpu::Wasm)) {
        return Err("WASM OS and CPU must be selected together".into());
    }
    if (parsed.progress.is_some()
        || parsed.inputs.is_some()
        || parsed.selected.is_some()
        || parsed.case_id.is_some())
        && parsed.dirs.len() != 1
    {
        return Err("case/progress/input selection requires one explicit project root".into());
    }
    if parsed.progress.as_ref().is_some_and(|p| p.exists()) {
        return Err("progress path must be new; preserve prior evidence".into());
    }
    parsed.stdlib = parsed
        .stdlib
        .canonicalize()
        .map_err(|e| format!("stdlib: {e}"))?;
    Ok(parsed)
}

/// The audit of every input root: the report so far, and which reviews and oracles matched.
struct Audit {
    args: Args,
    report: Report,
    used_reviews: Vec<bool>,
    used_oracles: Vec<bool>,
}

impl Audit {
    fn new(args: Args) -> Self {
        let host = jaic::sema::Options::host();
        let target = format!(
            "{:?}/{:?}",
            args.os.unwrap_or(host.os),
            args.cpu.unwrap_or(host.cpu)
        );
        let report = Report {
            schema: 1,
            build_id: option_env!("JAI_IDE_AUDIT_REVISION")
                .unwrap_or("unverified-build")
                .into(),
            stdlib_id: args.stdlib_id.clone(),
            target,
            runtime_support: args.runtime,
            redacted: false,
            scope: "explicit-input-subset; whole-corpus only with coordinator completeness ledger"
                .into(),
            accepted: false,
            counts: Counts::default(),
            files: vec![],
            expected_inputs: 0,
            query_selection: args.selected.as_ref().map(|s| {
                let mut ids = s.iter().cloned().collect::<Vec<_>>();
                ids.sort();
                ids
            }),
        };
        Self {
            used_reviews: vec![false; args.manifest.reviews.len()],
            used_oracles: vec![false; args.manifest.oracles.len()],
            args,
            report,
        }
    }

    /// The Jai files of one input root: the explicit inputs, or everything under it.
    fn discover(&self, dir: &Path) -> Result<Vec<Result<PathBuf, String>>, String> {
        let mut files = Vec::new();
        if let Some(inputs) = &self.args.inputs {
            let mut inputs = inputs.clone();
            inputs.sort();
            if inputs.windows(2).any(|w| w[0] == w[1]) {
                return Err("duplicate explicit input".into());
            }
            for input in inputs {
                let relative = Path::new(&input);
                if relative.is_absolute()
                    || relative
                        .components()
                        .any(|c| matches!(c, std::path::Component::ParentDir))
                {
                    return Err("explicit inputs must be relative to the project root".into());
                }
                files.push(dir.join(relative).canonicalize().map_err(|e| e.to_string()));
            }
        } else {
            walk(dir, &mut HashSet::default(), &mut files);
        }
        if files.is_empty() {
            files.push(Err("input contains no Jai files".into()));
        }
        if let Some(selected) = &self.args.selected {
            let known: HashSet<_> = (0..files.len()).map(|i| format!("f{i:06}")).collect();
            if selected.is_empty() || !selected.is_subset(&known) {
                return Err("unknown/empty input selection".into());
            }
        }
        Ok(files)
    }

    /// Opens each file in the session. The ones that cannot be opened are recorded as failures.
    fn open_files(
        &mut self,
        session: &mut Session,
        id: &str,
        files: Vec<Result<PathBuf, String>>,
    ) -> Vec<(File, DocumentUri, String)> {
        let mut opened = Vec::new();
        for (index, path) in files.into_iter().enumerate() {
            let mut file = File {
                case_id: id.to_owned(),
                input_id: format!("f{index:06}"),
                path: None,
                fingerprint: None,
                status: "unexecuted".into(),
                identifiers: 0,
                syntax_tokens: 0,
                semantic: None,
                failures: vec![],
                gaps: vec![],
            };
            let path = match path {
                Ok(path) => path,
                Err(e) => {
                    file.failures.push(failure("discovery", e));
                    self.report.files.push(file);
                    continue;
                }
            };
            file.path = Some(path.to_string_lossy().into_owned());
            let text = match std::fs::read_to_string(&path) {
                Ok(text) => text,
                Err(e) => {
                    file.failures.push(failure("read", e.to_string()));
                    self.report.files.push(file);
                    continue;
                }
            };
            file.fingerprint = Some(report::fingerprint(&text));
            let document = match uri(&path) {
                Ok(uri) => uri,
                Err(e) => {
                    file.failures.push(failure("uri", e));
                    self.report.files.push(file);
                    continue;
                }
            };
            match session.open(document.clone(), 1, text.clone()) {
                Ok(()) => opened.push((file, document, text)),
                Err(e) => {
                    file.failures.push(failure("open", format!("{e:?}")));
                    self.report.files.push(file);
                }
            }
        }
        opened
    }

    /// Audits every file of one input root in a fresh session.
    fn audit_root(&mut self, root_index: usize, dir: PathBuf) -> Result<(), String> {
        let id = self
            .args
            .case_id
            .clone()
            .unwrap_or_else(|| format!("root{root_index:04}"));
        let files = self.discover(&dir)?;
        self.report.expected_inputs += files.len();
        let progress = self.args.progress.clone();
        checkpoint(
            progress.as_deref(),
            serde_json::json!({"kind":"header","build_id":self.report.build_id,"stdlib_id":self.report.stdlib_id,"target":self.report.target,"runtime_support":self.args.runtime,"case_id":id,"expected_inputs":files.len(),"selection":self.args.selected,"redacted":true}),
        )?;
        let mut env = environment(self.args.stdlib.clone(), self.args.runtime);
        let options = env.options;
        let (os, cpu) = (self.args.os, self.args.cpu);
        env.options = Box::new(move |path| {
            let mut options = options(path);
            jaic::target::select_os_cpu(&mut options, os, cpu);
            options
        });
        let mut session = Session::with_environment(Limits::default(), env);
        session.set_workspace_folders(vec![dir.canonicalize().unwrap_or(dir)]);
        let opened = self.open_files(&mut session, &id, files);
        for file in &self.report.files {
            checkpoint_file(progress.as_deref(), file)?;
        }
        let plan = report::fingerprint(
            &opened
                .iter()
                .map(|(file, _, _)| {
                    format!(
                        "{}:{}",
                        file.input_id,
                        file.fingerprint.as_deref().unwrap_or("unreadable")
                    )
                })
                .collect::<Vec<_>>()
                .join("\n"),
        );
        let destinations: std::collections::BTreeMap<_, _> = opened
            .iter()
            .map(|(file, uri, _)| (file.input_id.clone(), uri.as_str().to_owned()))
            .collect();
        checkpoint(
            progress.as_deref(),
            serde_json::json!({"kind":"plan","fingerprint":plan,"redacted":true}),
        )?;
        for (file, uri, text) in opened {
            if self
                .args
                .selected
                .as_ref()
                .is_some_and(|s| !s.contains(&file.input_id))
            {
                continue;
            }
            self.audit_file(&mut session, file, &uri, &text, &destinations)?;
        }
        Ok(())
    }

    /// Queries diagnostics, then hover and definition at every identifier of one file.
    fn audit_file(
        &mut self,
        session: &mut Session,
        mut file: File,
        uri: &DocumentUri,
        text: &str,
        destinations: &std::collections::BTreeMap<String, String>,
    ) -> Result<(), String> {
        let progress = self.args.progress.clone();
        checkpoint(
            progress.as_deref(),
            serde_json::json!({"kind":"start","input_id":file.input_id,"redacted":true}),
        )?;
        match session.diagnostics(uri) {
            Ok(diagnostics) => {
                for d in diagnostics.into_iter().filter(|d| {
                    d.severity == DiagnosticSeverity::Error || d.code == DiagnosticCode::Limit
                }) {
                    file.failures.push(Failure {
                        kind: format!("{:?}", d.code).to_lowercase(),
                        line: Some(d.range.start.line),
                        character: Some(d.range.start.character),
                        diagnostic: Some(d.message),
                    });
                }
            }
            Err(e) => file.failures.push(failure("diagnostics", format!("{e:?}"))),
        }
        let tokens = match jaic::lexer::lex(jaic::source::FileId(0), text) {
            Ok(tokens) => tokens,
            Err(e) => {
                file.failures.push(failure("lex", e.message));
                checkpoint_file(progress.as_deref(), &file)?;
                self.report.files.push(file);
                return Ok(());
            }
        };
        file.status = "audited".into();
        self.audit_tokens(session, &mut file, uri, text, &tokens, destinations)?;
        file.semantic = match session.semantic_coverage(uri) {
            Ok(coverage) => coverage,
            Err(e) => {
                file.failures
                    .push(failure("coverage_query", format!("{e:?}")));
                None
            }
        };
        if file.semantic.as_ref().is_none_or(|s| !s.source_loaded) {
            file.failures.push(failure(
                "semantic_unreached",
                "input not loaded into a semantic compiler context".into(),
            ));
        } else if file
            .semantic
            .as_ref()
            .is_some_and(|s| !s.program_complete || s.program_errors > 0)
        {
            file.failures.push(failure(
                "context_incomplete",
                "analysis context has compilation errors; local diagnostics alone are insufficient"
                    .into(),
            ));
        }
        checkpoint_file(progress.as_deref(), &file)?;
        self.report.files.push(file);
        Ok(())
    }

    /// Asks for hover and definition at each identifier and records what is missing.
    fn audit_tokens(
        &mut self,
        session: &mut Session,
        file: &mut File,
        uri: &DocumentUri,
        text: &str,
        tokens: &[jaic::lexer::Token],
        destinations: &std::collections::BTreeMap<String, String>,
    ) -> Result<(), String> {
        let progress = self.args.progress.clone();
        let lines = jaic::source::LineIndex::new(text);
        for (index, token) in tokens.iter().enumerate() {
            if matches!(&token.tok,Tok::Ident(name) if KEYWORDS.contains(&name.as_str()))
                || using_modifier(tokens, index)
            {
                file.syntax_tokens += 1;
                continue;
            }
            let (name, role) = match &token.tok {
                Tok::Ident(_)
                    if index >= 2
                        && matches!(tokens[index - 1].tok, Tok::Punct(P::Comma))
                        && matches!(tokens[index - 2].tok, Tok::Directive(_)) =>
                {
                    file.syntax_tokens += 1;
                    continue;
                }
                Tok::Ident(name) if !KEYWORDS.contains(&name.as_str()) => {
                    (name.to_string(), role(tokens, index))
                }
                Tok::Directive(name) => (format!("#{name}"), Role::Directive),
                _ => continue,
            };
            file.identifiers += 1;
            // Probe the first codepoint, never start+1 (which may split UTF8).
            let at = position(text, &lines, token.span.start as usize);
            if file.identifiers % 256 == 1 {
                checkpoint(
                    progress.as_deref(),
                    serde_json::json!({"kind":"cursor","input_id":file.input_id,"line":at.line,"character":at.character,"queried_before":file.identifiers-1,"redacted":true}),
                )?;
            }
            let hover = match session.hover(uri, at) {
                Ok(h) => h.is_some_and(|h| !h.contents.value.contains("Source syntax declaration")),
                Err(e) => {
                    file.failures
                        .push(failure("hover_request", format!("{e:?}")));
                    false
                }
            };
            let locations = match session.definition(uri, at) {
                Ok(d) => d,
                Err(e) => {
                    file.failures
                        .push(failure("definition_request", format!("{e:?}")));
                    vec![]
                }
            };
            let definitions: Vec<_> = locations
                .into_iter()
                .map(|l| Target {
                    uri: l.uri,
                    line: l.range.start.line,
                    character: l.range.start.character,
                })
                .collect();
            let definition = role == Role::Directive || !definitions.is_empty();
            let mut wrong = false;
            for (i, oracle) in self
                .args
                .manifest
                .oracles
                .iter()
                .enumerate()
                .filter(|(_, o)| {
                    o.case_id == file.case_id
                        && o.input_id == file.input_id
                        && Some(&o.fingerprint) == file.fingerprint.as_ref()
                        && o.line == at.line
                        && o.character == at.character
                })
            {
                self.used_oracles[i] = true;
                let expected = oracle
                    .definitions
                    .iter()
                    .cloned()
                    .map(|mut target| {
                        if let Some(id) = target.uri.strip_prefix("input:")
                            && let Some(uri) = destinations.get(id)
                        {
                            target.uri = uri.clone();
                        }
                        target
                    })
                    .collect::<Vec<_>>();
                wrong = !report::definitions_match(&expected, &definitions);
            }
            if hover && definition && !wrong {
                continue;
            }
            let missing = Missing {
                hover: !hover,
                definition: !definition,
                wrong_definition: wrong,
            };
            let matches: Vec<_> = self
                .args
                .manifest
                .reviews
                .iter()
                .enumerate()
                .filter(|(_, r)| {
                    r.case_id == file.case_id
                        && r.input_id == file.input_id
                        && Some(&r.fingerprint) == file.fingerprint.as_ref()
                        && r.line == at.line
                        && r.character == at.character
                        && r.missing == missing
                        && !r.evidence.trim().is_empty()
                })
                .collect();
            let classification = if let [(i, review)] = matches.as_slice() {
                self.used_reviews[*i] = true;
                Some(review.reason.clone())
            } else {
                None
            };
            file.gaps.push(Gap {
                line: at.line,
                character: at.character,
                role: format!("{role:?}").to_lowercase(),
                name: Some(name),
                missing,
                classification,
                definitions,
            });
        }
        Ok(())
    }
}

fn run() -> Result<(Report, bool, bool), String> {
    let mut audit = Audit::new(parse_args()?);
    for (root_index, dir) in std::mem::take(&mut audit.args.dirs).into_iter().enumerate() {
        audit.audit_root(root_index, dir)?;
    }
    audit.report.finish(
        &audit.args.manifest.reviews,
        &audit.used_reviews,
        audit.used_oracles.iter().filter(|used| !**used).count(),
    );
    if audit.args.redacted {
        audit.report.redact();
    }
    Ok((audit.report, audit.args.json, audit.args.strict))
}

pub fn main() {
    match run() {
        Ok((report, json, strict)) => {
            if json {
                println!(
                    "{}",
                    serde_json::to_string_pretty(&report).expect("JSON report")
                );
            } else {
                println!(
                    "{} files, {} identifiers, {} gaps ({} unexpected), {} failures, {} unexecuted; accepted={}",
                    report.counts.files,
                    report.counts.identifiers,
                    report.counts.gaps,
                    report.counts.unexpected_gaps,
                    report.counts.failures,
                    report.counts.unexecuted_files,
                    report.accepted
                );
            }
            if strict && !report.accepted {
                std::process::exit(1);
            }
        }
        Err(e) => {
            eprintln!("{e}");
            std::process::exit(2);
        }
    }
}
