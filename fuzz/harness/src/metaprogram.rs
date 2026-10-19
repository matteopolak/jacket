//! Metaprograms from the fuzzer's bytes: a `#run` that creates workspaces, sets their build
//! options, intercepts their messages and feeds them generated sources, with `#insert` and
//! `compiler_get_nodes` mixed in. Random bytes seldom reach the workspace state
//! machine (create, intercept, add sources, wait for messages, end, destroy); this grammar does,
//! including the misuse a real metaprogram gets wrong: waiting without intercepting, ending twice,
//! using a destroyed workspace, adding sources after the program is checked.
//!
//! The program runs like [`crate::interp`]: the playground sandbox and its block budget.
use arbitrary::{Result, Unstructured};
use std::fmt::Write as _;

/// Programs a workspace is given (`%` is replaced by a number from the input).
const SOURCES: &[&str] = &[
    "main :: () {}",
    "#import \"Basic\";\nmain :: () { print(\"%\\n\", %); }",
    "f :: (x: int) -> int { return x * % + 1; }\nmain :: () { f(2); }",
    "#run { a := %; }\nmain :: () {}",
    "S :: struct { a: int = %; }\nmain :: () { s: S; }",
    "main :: () { x: int = \"text\"; }",
    "main :: (",
    "#placeholder P;\nmain :: () { }",
    "main :: () { #insert \"y := %;\"; z := y; }",
    "T :: struct (N: int) { a: [N] int; }\nmain :: () { t: T(%); }",
    "#import \"Compiler\";\nmain :: () { w := compiler_create_workspace(\"inner\"); }",
    "#run compiler_report(\"from a run\", mode = .WARNING);\nmain :: () {}",
    "",
];

/// Statements that change the build options of workspace `w`.
const OPTIONS: &[&str] = &[
    "opts.output_type = .NO_OUTPUT;",
    "opts.output_executable_name = \"generated\";",
    "opts.runtime_support_definitions = .OMIT;",
    "opts.backtrace_on_crash = .OFF;",
    "opts.import_path = .[\"modules\"];",
    "opts.text_output_flags = 0;",
];

/// Message loops; `w` is the workspace and `m` the message.
const LOOPS: &[&str] = &[
    "while true { m := compiler_wait_for_message(); if !m break; if m.kind == .COMPLETE break; }",
    "done := false; files, phases, bodies: int; while !done { m := compiler_wait_for_message(); if !m break; if m.kind == { case .FILE; files += 1; case .PHASE; phases += 1; case .TYPECHECKED; t := cast(*Message_Typechecked) m; for t.procedure_bodies bodies += 1; case .COMPLETE; done = true; } }",
    "count := 0; while count < 40 { count += 1; m := compiler_wait_for_message(); if !m break; if m.kind == .PHASE { p := cast(*Message_Phase) m; if p.phase == .TYPECHECKED_ALL_WE_CAN add_build_string(\"late :: () {}\", w); } if m.kind == .COMPLETE break; }",
    "m := compiler_wait_for_message();",
    "",
];

struct Gen<'a> {
    u: Unstructured<'a>,
    out: String,
}

/// Render the metaprogram chosen by `data`.
pub fn program(data: &[u8]) -> String {
    let mut g = Gen {
        u: Unstructured::new(data),
        out: String::new(),
    };
    let _ = g.program();
    g.out
}

fn quoted(text: &str) -> String {
    format!(
        "\"{}\"",
        text.replace('\\', "\\\\")
            .replace('"', "\\\"")
            .replace('\n', "\\n")
    )
}

impl Gen<'_> {
    fn line(&mut self, text: &str) {
        let _ = writeln!(self.out, "{text}");
    }

    fn pick<T: Copy>(&mut self, items: &[T]) -> Result<T> {
        Ok(*self.u.choose(items)?)
    }

    fn number(&mut self) -> Result<String> {
        Ok(match self.u.int_in_range(0..=5)? {
            0 => "0".into(),
            1 => "-1".into(),
            2 => "9223372036854775807".into(),
            _ => self.u.int_in_range(0..=1000)?.to_string(),
        })
    }

    fn source(&mut self) -> Result<String> {
        let text = self.pick(SOURCES)?;
        let n = self.number()?;
        Ok(text.replace('%', &n))
    }

    fn program(&mut self) -> Result<()> {
        self.line("#import \"Basic\";");
        self.line("#import \"Compiler\";");
        let n = self.number()?;
        self.line(&format!("K :: {n};"));
        self.line("make_source :: (name: string) -> string { return tprint(\"% :: () -> int { return %; }\", name, K); }");
        self.line("build :: () {");
        let workspaces = self.u.int_in_range(1..=2)?;
        for i in 0..workspaces {
            let name = format!("w{i}");
            self.workspace(&name)?;
        }
        self.line("}");
        match self.u.int_in_range(0..=4)? {
            0 => self.line("#run build();"),
            1 => self.line("#run { build(); build(); }"),
            2 => {
                self.line("#run build();");
                self.line("main :: () { #insert #run tprint(\"x := %;\", 1); }");
            }
            _ => self.line("#run build();"),
        }
        if !self.out.contains("main :: ()") {
            self.line("main :: () {}");
        }
        Ok(())
    }

    fn workspace(&mut self, w: &str) -> Result<()> {
        let name = quoted(&format!("generated {w}"));
        self.line("    {");
        self.line(&format!("    {w} := compiler_create_workspace({name});"));
        if self.u.ratio(1, 2)? {
            self.line(&format!("    opts := get_build_options({w});"));
            for _ in 0..self.u.int_in_range(0..=3)? {
                let option = self.pick(OPTIONS)?;
                self.line(&format!("    {option}"));
            }
            self.line(&format!("    set_build_options(opts, {w});"));
        }
        let intercept = !self.u.ratio(1, 8)?;
        if intercept {
            self.line(&format!("    compiler_begin_intercept({w});"));
        }
        for _ in 0..self.u.int_in_range(0..=3)? {
            let source = self.source()?;
            let target = if self.u.ratio(1, 10)? {
                "-1"
            } else {
                w
            };
            self.line(&format!(
                "    add_build_string({}, {target});",
                quoted(&source)
            ));
        }
        if self.u.ratio(1, 4)? {
            self.line(&format!(
                "    add_build_string(make_source(\"generated_proc\"), {w});"
            ));
        }
        if self.u.ratio(1, 4)? {
            let code = self.pick(&["1 + 2", "a := b * 3", "if x { y(); }", "f(1, 2)"])?;
            self.line(&format!(
                "    {{ root, exprs := compiler_get_nodes(#code {code}); print(\"% %\\n\", root.kind, exprs.count); }}"
            ));
        }
        let looping = self.pick(LOOPS)?;
        self.line(&format!("    w := {w};"));
        self.line(&format!("    {looping}"));
        if intercept && !self.u.ratio(1, 6)? {
            self.line(&format!("    compiler_end_intercept({w});"));
            if self.u.ratio(1, 6)? {
                self.line(&format!("    compiler_end_intercept({w});"));
            }
        }
        if self.u.ratio(1, 4)? {
            self.line(&format!("    compiler_destroy_workspace({w});"));
            if self.u.ratio(1, 4)? {
                self.line(&format!("    add_build_string(\"after :: () {{}}\", {w});"));
            }
        }
        self.line("    }");
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::program;

    #[test]
    fn empty_input_renders_a_program() {
        assert!(program(&[]).contains("build ::"));
    }

    #[test]
    fn varied_inputs_render_varied_programs() {
        let a = program(&[1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16]);
        let b = program(&[
            200, 100, 50, 25, 12, 6, 3, 1, 0, 255, 254, 253, 17, 99, 7, 1,
        ]);
        assert_ne!(a, b);
    }
}
