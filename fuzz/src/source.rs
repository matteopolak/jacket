use crate::{
    Bytes, MAX_PARSE_TOKENS, MAX_RECURSIVE_TOKENS, MAX_SOURCE_BYTES, MAX_VIRTUAL_FILES, limits,
};
use jai_lexer::{Keyword, Kind, Punct};
use jai_source::{Diagnostic, Span};
use std::borrow::Cow;

fn source(data: &[u8]) -> Option<Cow<'_, str>> {
    if data.len() > MAX_SOURCE_BYTES {
        return None;
    }
    let text = jai_lexer::decode_source(data).ok()?;
    (text.len() <= MAX_SOURCE_BYTES).then_some(text)
}

fn span(span: Span, source: &str) {
    assert!(span.start <= span.end && span.end <= source.len());
    assert!(source.is_char_boundary(span.start) && source.is_char_boundary(span.end));
}

fn diagnostic(error: &Diagnostic, source: &str) {
    span(error.span, source);
    let _ = error.render("authored-fuzz-input.jai", source);
}

pub fn lexer_utf8(data: &[u8]) {
    let Some(source) = source(data) else {
        return;
    };
    match jai_lexer::lex(&source) {
        Ok(tokens) => {
            let mut previous = 0;
            for token in &tokens {
                span(token.span, &source);
                assert!(token.span.start >= previous);
                previous = token.span.end;
                let _ = token.spelling(&source);
            }
            assert_eq!(tokens, jai_lexer::lex(&source).unwrap());
        }
        Err(error) => diagnostic(&error, &source),
    }
}

// Conservative harness admission prevents recursive parser work from escaping
// the requested bounded campaign. Rejected inputs still belong to lexer fuzzing.
fn parse_admitted(source: &str) -> bool {
    let Ok(tokens) = jai_lexer::lex(source) else {
        return true;
    };
    if tokens.len() > MAX_PARSE_TOKENS {
        return false;
    }
    let recursive = tokens
        .iter()
        .filter(|token| {
            matches!(
                token.kind,
                Kind::Punctuation(
                    Punct::OpenParen
                        | Punct::OpenBrace
                        | Punct::OpenBracket
                        | Punct::StructLiteral
                        | Punct::ArrayLiteral
                        | Punct::Sub
                        | Punct::Not
                        | Punct::Complement
                        | Punct::Mul
                ) | Kind::Keyword(
                    Keyword::If
                        | Keyword::Ifx
                        | Keyword::While
                        | Keyword::For
                        | Keyword::Struct
                        | Keyword::Union
                        | Keyword::Cast
                        | Keyword::PushContext
                )
            )
        })
        .count();
    recursive <= MAX_RECURSIVE_TOKENS
}

pub fn parser(data: &[u8]) {
    let Some(source) = source(data) else {
        return;
    };
    if !parse_admitted(&source) {
        return;
    }
    if let Err(error) = jai_syntax::parse(&source) {
        diagnostic(&error, &source);
    }
}

fn graph_program(bundle: &jai_runtime::SourceBundle, path: &std::path::Path) {
    let target = jai_runtime::browser_target();
    let Ok(graph) = jai_modules::ModuleGraph::load_with_target(
        path,
        jai_modules::GraphOptions::default(),
        bundle,
        target.clone(),
    ) else {
        return;
    };
    let options = jai_sema::ResolveOptions {
        layout: Some(target.layout),
        target: Some(target.clone()),
        compile_time_limits: limits(),
        compiler: Some(jai_sema::CompilerBindingContext::from_graph(
            &graph,
            &[],
            jai_vm::WorkspaceId::from_raw(1).unwrap(),
        )),
        ..Default::default()
    };
    match jai_sema::resolve_graph_with_options(&graph, &options, &mut jai_vm::NoEffects) {
        Ok(program) => {
            let Ok(mut vm) = jai_vm::Vm::new_with_execution_phase(
                &program,
                jai_vm::NoEffects,
                limits(),
                jai_vm::ByteTarget::from(&target),
                jai_vm::ExecutionPhase::Runtime,
            ) else {
                return;
            };
            let entry = match program.entry() {
                jai_ir::EntryPoint::Int(id) | jai_ir::EntryPoint::Void(id) => id,
            };
            let _ = vm.execute(entry, vec![]);
        }
        Err(error) => {
            let record = graph
                .sources()
                .get(error.location.source)
                .expect("semantic rejection must retain an actual source identity");
            span(error.location.span, record.text());
            let _ = error.render(graph.sources());
        }
    }
}

pub fn module_vfs(data: &[u8]) {
    if data.len() > MAX_SOURCE_BYTES {
        return;
    }
    let mut bundle = jai_runtime::SourceBundle::new(MAX_SOURCE_BYTES);
    let mut entry = None;
    for (index, bytes) in data
        .split(|byte| *byte == 0)
        .take(MAX_VIRTUAL_FILES)
        .enumerate()
    {
        let Some(source) = source(bytes) else {
            return;
        };
        if !parse_admitted(&source) {
            return;
        }
        let name = if index == 0 {
            "main.jai".to_owned()
        } else {
            format!("part-{index}.jai")
        };
        let Ok(path) = bundle.insert(name, source.as_bytes().to_vec()) else {
            return;
        };
        if index == 0 {
            entry = Some(path);
        }
    }
    if let Some(path) = entry {
        graph_program(&bundle, &path);
    }
}

pub fn constant_sema(data: &[u8]) {
    let mut bytes = Bytes::new(data);
    let count = usize::from(bytes.byte() % 16);
    let mut expected = bytes.signed();
    let mut source = format!("C0 :: {expected};\n");
    for index in 0..count {
        let delta = bytes.signed();
        let condition = bytes.byte() & 1 == 0;
        source.push_str(&format!(
            "C{} :: ifx {condition} then C{index} + ({delta}) else C{index} - ({delta});\n",
            index + 1
        ));
        expected += if condition {
            delta
        } else {
            -delta
        };
    }
    let iterations = bytes.byte() % 9;
    expected += i128::from(iterations) * (i128::from(iterations) + 1) / 2;
    source.push_str(&format!("adjust :: (n:int)->int {{for i:1..{iterations} n+=i;return n;}} main::()->int {{return #run adjust(C{count});}}"));
    let options = jai_runtime::Options {
        target: jai_runtime::browser_target(),
        limits: limits(),
        ..Default::default()
    };
    let script = jai_runtime::Script::from_source(&source, options)
        .expect("structured pure constant program must resolve");
    assert_eq!(i128::from(script.run(&[]).unwrap().exit_code), expected);
    assert_eq!(i128::from(script.run(&[]).unwrap().exit_code), expected);
}
