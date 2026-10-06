//! Grammar-based program generator: fuzzer bytes, read through `arbitrary::Unstructured`, choose
//! the productions of a small type-directed Jai grammar. Random bytes rarely get past the
//! parser; these programs mostly parse and type-check, so mutations land in sema, `#run`, macro
//! expansion, polymorphism and the interpreter instead.
//!
//! The generator is typed: it tracks the locals in scope and asks for an expression of a given
//! type, so most programs are valid. A small fraction of choices deliberately pick an expression
//! of the wrong type or an undeclared name to keep the error paths covered.
//!
//! Safety of the interpreted program matters here (the interpreter uses real memory): nothing
//! generated forms a pointer from an integer, leaves a pointer uninitialized with `---`, or keeps a
//! pointer past its scope. Pointers only come from `*local`, and null dereferences trap.
use arbitrary::{Result, Unstructured};
use std::fmt::Write as _;

/// The value types the generator can produce expressions for.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Ty {
    Int,
    S32,
    U8,
    Float,
    F32,
    Bool,
    Str,
    /// `[4] int`
    Array,
    /// `[..] int`
    Dynamic,
    /// `[] int`
    View,
    /// `S<n>`
    Struct(usize),
    /// `E<n>`
    Enum(usize),
    /// `*int`
    Pointer,
    /// `Box(int)`, a polymorphic struct instance
    Boxed,
}

const SCALARS: &[Ty] = &[
    Ty::Int,
    Ty::S32,
    Ty::U8,
    Ty::Float,
    Ty::F32,
    Ty::Bool,
    Ty::Str,
];

struct Proc {
    name: String,
    params: Vec<Ty>,
    ret: Option<Ty>,
}

struct StructDecl {
    fields: Vec<(String, Ty)>,
}

struct Gen<'a> {
    u: Unstructured<'a>,
    out: String,
    indent: usize,
    basic: bool,
    /// Whether this program sometimes uses a value of the wrong type (error paths).
    chaos: bool,
    structs: Vec<StructDecl>,
    enums: Vec<usize>,
    procs: Vec<Proc>,
    constants: Vec<String>,
    /// Macro numbers and whether each takes a `Code` argument.
    macros: Vec<(usize, bool)>,
    /// What main mentions for each `edge_decl` (a name or an instance).
    edge_names: Vec<String>,
    /// Locals in scope, innermost last; each block remembers how many to drop.
    locals: Vec<(String, Ty)>,
    next_local: usize,
    loop_depth: usize,
    /// Remaining expression nodes; generation falls back to leaves when it runs out.
    fuel: usize,
    /// Return type of the procedure being generated.
    returns: Option<Ty>,
}

/// Render the program chosen by `data`. Exhausted input just ends the program early.
pub fn program(data: &[u8]) -> String {
    let mut g = Gen {
        u: Unstructured::new(data),
        out: String::new(),
        indent: 0,
        basic: false,
        chaos: false,
        structs: Vec::new(),
        enums: Vec::new(),
        procs: Vec::new(),
        constants: Vec::new(),
        macros: Vec::new(),
        edge_names: Vec::new(),
        locals: Vec::new(),
        next_local: 0,
        loop_depth: 0,
        fuel: 400,
        returns: None,
    };
    let _ = g.program();
    g.out
}

impl Gen<'_> {
    fn line(&mut self, text: &str) {
        for _ in 0..self.indent {
            self.out.push_str("    ");
        }
        self.out.push_str(text);
        self.out.push('\n');
    }

    fn chance(&mut self, num: u32, den: u32) -> Result<bool> {
        self.u.ratio(num, den)
    }

    fn pick<'b, T>(&mut self, items: &'b [T]) -> Result<&'b T> {
        self.u.choose(items)
    }

    fn ty_name(&self, ty: Ty) -> String {
        match ty {
            Ty::Int => "int".into(),
            Ty::S32 => "s32".into(),
            Ty::U8 => "u8".into(),
            Ty::Float => "float".into(),
            Ty::F32 => "float32".into(),
            Ty::Bool => "bool".into(),
            Ty::Str => "string".into(),
            Ty::Array => "[4] int".into(),
            Ty::Dynamic => "[..] int".into(),
            Ty::View => "[] int".into(),
            Ty::Struct(i) => format!("S{i}"),
            Ty::Enum(i) => format!("E{i}"),
            Ty::Pointer => "*int".into(),
            Ty::Boxed => "Box(int)".into(),
        }
    }

    fn any_ty(&mut self) -> Result<Ty> {
        Ok(match self.u.int_in_range(0..=13)? {
            0..=6 => *self.pick(SCALARS)?,
            7 => Ty::Array,
            8 if self.basic => Ty::Dynamic,
            9 => Ty::View,
            10 if !self.structs.is_empty() => Ty::Struct(self.u.choose_index(self.structs.len())?),
            11 if !self.enums.is_empty() => Ty::Enum(self.u.choose_index(self.enums.len())?),
            12 => Ty::Pointer,
            13 => Ty::Boxed,
            _ => Ty::Int,
        })
    }

    fn program(&mut self) -> Result<()> {
        self.basic = self.chance(1, 3)?;
        self.chaos = self.chance(1, 8)?;
        if self.basic {
            self.line("#import \"Basic\";");
        }
        self.line("Box :: struct (T: Type) { v: T; }");
        for _ in 0..self.u.int_in_range(0..=3)? {
            self.enum_decl()?;
        }
        for _ in 0..self.u.int_in_range(0..=3)? {
            self.struct_decl()?;
        }
        let decls = self.u.int_in_range(0..=10)?;
        for _ in 0..decls {
            if self.u.is_empty() {
                break;
            }
            self.top_decl()?;
        }
        self.returns = None;
        self.line("main :: () {");
        self.indent += 1;
        self.block_body(6)?;
        for name in self.edge_names.clone() {
            self.line(&format!("_ := type_of({name});"));
        }
        // Sema is demand-driven: only what main reaches is checked, so call every procedure.
        for i in 0..self.procs.len() {
            self.fuel = 20;
            let ret = self.procs[i].ret;
            let call = self.call_proc(i, 2)?;
            if ret.is_some() && self.basic {
                self.line(&format!("print(\"%\\n\", {call});"));
            } else {
                self.line(&format!("{call};"));
            }
        }
        self.indent -= 1;
        self.line("}");
        Ok(())
    }

    fn enum_decl(&mut self) -> Result<()> {
        let i = self.enums.len();
        let count = self.u.int_in_range(1..=5)?;
        let kind = *self.pick(&["enum", "enum u8", "enum_flags", "enum s16"])?;
        let mut text = format!("E{i} :: {kind} {{");
        for m in 0..count {
            if self.chance(1, 4)? && !kind.starts_with("enum_flags") {
                let _ = write!(text, " M{m} :: {};", self.u.int_in_range(-3..=9)?);
            } else {
                let _ = write!(text, " M{m};");
            }
        }
        text.push_str(" }");
        self.line(&text);
        self.enums.push(count);
        Ok(())
    }

    fn struct_decl(&mut self) -> Result<()> {
        let i = self.structs.len();
        let count = self.u.int_in_range(1..=4)?;
        let mut fields = Vec::new();
        let mut text = format!("S{i} :: struct {{");
        if i > 0 && self.chance(1, 4)? {
            let base = self.u.choose_index(i)?;
            let _ = write!(text, " using base: S{base};");
            for (name, ty) in &self.structs[base].fields {
                fields.push((name.clone(), *ty));
            }
        }
        for f in 0..count {
            let ty = match self.u.int_in_range(0..=9)? {
                0..=6 => *self.pick(SCALARS)?,
                7 => Ty::Array,
                8 if i > 0 => Ty::Struct(self.u.choose_index(i)?),
                9 if !self.enums.is_empty() => Ty::Enum(self.u.choose_index(self.enums.len())?),
                _ => Ty::Int,
            };
            let name = format!("f{i}_{f}");
            let _ = write!(text, " {name}: {}", self.ty_name(ty));
            if SCALARS.contains(&ty) && self.chance(1, 3)? {
                self.fuel = 6;
                let init = self.expr(ty, 1)?;
                let _ = write!(text, " = {init}");
            }
            text.push(';');
            fields.push((name, ty));
        }
        if self.chance(1, 6)? {
            text.push_str(" #place f");
            let _ = write!(text, "{i}_0; overlay: u8;");
        }
        text.push_str(" }");
        self.line(&text);
        self.structs.push(StructDecl {
            fields,
        });
        Ok(())
    }

    fn top_decl(&mut self) -> Result<()> {
        match self.u.int_in_range(0..=10)? {
            0..=3 => self.proc_decl(),
            10 => self.edge_decl(),
            4 => {
                let i = self.constants.len();
                self.fuel = 12;
                let value = if self.chance(1, 3)? && !self.procs.is_empty() {
                    format!("#run {}", self.call(Ty::Int, 2)?)
                } else {
                    self.expr(Ty::Int, 2)?
                };
                self.line(&format!("K{i} :: {value};"));
                self.constants.push(format!("K{i}"));
                Ok(())
            }
            5 => {
                // A polymorphic procedure, called from generated code with ints or floats.
                let n = self.procs.len();
                let body = *self.pick(&[
                    "return p0;",
                    "return p1;",
                    "return p0 + p1;",
                    "return p0 * p1 - p0;",
                    "if p0 < p1 return p1; return p0;",
                ])?;
                self.line(&format!("P{n} :: (p0: $T, p1: T) -> T {{ {body} }}"));
                self.procs.push(Proc {
                    name: format!("P{n}"),
                    params: vec![Ty::Int, Ty::Int],
                    ret: Some(Ty::Int),
                });
                Ok(())
            }
            6 => {
                // Macros: inserted code, and a backtick reaching the caller's `it`-free local.
                let n = self.macros.len();
                let takes_code = self.chance(1, 2)?;
                self.macros.push((n, takes_code));
                if takes_code {
                    self.line(&format!(
                        "M{n} :: (c: Code) #expand {{ #insert c; #insert c; }}"
                    ));
                } else {
                    self.line(&format!(
                        "M{n} :: (x: int) -> int #expand {{ defer `result_{n} += 1; return x * 2; }}"
                    ));
                }
                Ok(())
            }
            7 => {
                self.fuel = 12;
                let cond = self.expr(Ty::Bool, 2)?;
                self.line(&format!("#if {cond} {{"));
                self.indent += 1;
                self.proc_decl()?;
                self.indent -= 1;
                self.line("} else {");
                self.indent += 1;
                // Same name, so either branch defines it.
                let last = self.procs.pop();
                if let Some(p) = &last {
                    let params: Vec<String> = p
                        .params
                        .iter()
                        .enumerate()
                        .map(|(i, t)| format!("p{i}: {}", self.ty_name(*t)))
                        .collect();
                    let ret = p
                        .ret
                        .map(|t| format!(" -> {}", self.ty_name(t)))
                        .unwrap_or_default();
                    let value = match p.ret {
                        Some(t) => {
                            self.fuel = 8;
                            format!(" return {};", self.leaf(t)?)
                        }
                        None => String::new(),
                    };
                    self.line(&format!(
                        "{} :: ({}){ret} {{{value} }}",
                        p.name,
                        params.join(", ")
                    ));
                }
                self.indent -= 1;
                self.line("}");
                self.procs.extend(last);
                Ok(())
            }
            8 => {
                self.line("#run {");
                self.indent += 1;
                self.returns = None;
                self.block_body(3)?;
                self.indent -= 1;
                self.line("}");
                Ok(())
            }
            _ => {
                let i = self.constants.len();
                self.fuel = 10;
                let t = *self.pick(SCALARS)?;
                let v = self.expr(t, 2)?;
                let name = format!("G{i}");
                self.line(&format!("{name}: {} = {v};", self.ty_name(t)));
                Ok(())
            }
        }
    }

    /// A declaration from the shapes that broke the compiler before (each with numbers from the
    /// input): polymorphic recursion, `using` pointer cycles, self-inserting strings, cyclic
    /// compile-time pointers, oversized types, odd alignment, extreme constant arithmetic.
    fn edge_decl(&mut self) -> Result<()> {
        let n = self.edge_number()?;
        let m = self.edge_number()?;
        let k = self.edge_names.len();
        let (name, text) = match self.u.int_in_range(0..=11)? {
            0 => (
                format!("X{k}(int)"),
                format!("X{k} :: struct (T: Type) {{ p: *X{k}(*T); v: [{n}] u8; }}"),
            ),
            1 => (
                format!("R{k}(0)"),
                format!(
                    "R{k} :: ($N: int) -> int {{ #if N > {m} return N; else return R{k}({n} + N); }}"
                ),
            ),
            2 => (
                format!("U{k}"),
                format!(
                    "U{k} :: struct {{ using a: *V{k}; x: [{n}] int; }}\nV{k} :: struct {{ using b: *U{k}; y: int #align {m}; }}"
                ),
            ),
            3 => (format!("I{k}"), format!("I{k} :: \"#insert I{k};\";")),
            4 => (
                format!("RING{k}"),
                format!(
                    "C{k} :: struct {{ next: *C{k}; v: int; }}\nmake{k} :: () -> *C{k} {{ c := cast(*C{k}) alloc(size_of(C{k})); c.next = c; c.v = {n}; return c; }}\nRING{k} :: #run make{k}();"
                ),
            ),
            5 => (format!("A{k}"), format!("A{k} :: [{n}] [{m}] u8;")),
            6 => (
                format!("S{k}x"),
                format!("S{k}x :: struct {{ a: u8 #align {n}; b: [{m}] u16; }} #no_padding"),
            ),
            7 => (
                format!("K{k}x"),
                format!("K{k}x :: ({n} << {m}) / ({m} % ({n} | 1)) - (-{n} >> {m});"),
            ),
            8 => (
                format!("E{k}x"),
                format!("E{k}x :: enum u8 {{ A :: {n}; B :: {m}; C; }}"),
            ),
            9 => (format!("T{k}x"), format!("T{k}x :: #run cast(Type) {n};")),
            10 => (
                format!("D{k}x"),
                format!("D{k}x :: #type,distinct [{n}] D{k}y;\nD{k}y :: #type,distinct *D{k}x;"),
            ),
            _ => (
                format!("Q{k}x"),
                format!(
                    "Q{k}x :: \"{}\";\n#insert Q{k}x;",
                    "x".repeat((n.unsigned_abs() % 64) as usize)
                ),
            ),
        };
        self.line(&text);
        // Main mentions it, or demand-driven sema never looks at it.
        self.edge_names.push(name);
        Ok(())
    }

    fn edge_number(&mut self) -> Result<i64> {
        Ok(match self.u.int_in_range(0..=7)? {
            0 => 0,
            1 => -1,
            2 => self.u.int_in_range(1..=8)?,
            3 => 1 << self.u.int_in_range(0..=62)?,
            4 => i64::MAX,
            5 => i64::MIN + 1,
            6 => -self.u.int_in_range(1..=128)?,
            _ => self.u.int_in_range(0..=70)?,
        })
    }

    fn proc_decl(&mut self) -> Result<()> {
        let n = self.procs.len();
        let count = self.u.int_in_range(0..=3)?;
        let mut params = Vec::new();
        for _ in 0..count {
            params.push(self.any_ty()?);
        }
        let ret = if self.chance(2, 3)? {
            Some(*self.pick(SCALARS)?)
        } else {
            None
        };
        let text_params: Vec<String> = params
            .iter()
            .enumerate()
            .map(|(i, t)| format!("p{i}: {}", self.ty_name(*t)))
            .collect();
        let ret_text = ret
            .map(|t| format!(" -> {}", self.ty_name(t)))
            .unwrap_or_default();
        let name = format!("f{n}");
        self.line(&format!(
            "{name} :: ({}){ret_text} {{",
            text_params.join(", ")
        ));
        // Recursion is allowed (calls to itself are bounded by the interpreter's budget).
        self.procs.push(Proc {
            name,
            params: params.clone(),
            ret,
        });
        let saved = std::mem::take(&mut self.locals);
        for (i, t) in params.iter().enumerate() {
            self.locals.push((format!("p{i}"), *t));
        }
        self.returns = ret;
        self.indent += 1;
        self.block_body(4)?;
        if let Some(t) = ret {
            self.fuel = 10;
            let v = self.expr(t, 2)?;
            self.line(&format!("return {v};"));
        }
        self.indent -= 1;
        self.locals = saved;
        self.returns = None;
        self.line("}");
        Ok(())
    }

    fn block_body(&mut self, max: u32) -> Result<()> {
        let mark = self.locals.len();
        let count = self.u.int_in_range(0..=max)?;
        for _ in 0..count {
            if self.u.is_empty() {
                break;
            }
            self.stmt(3)?;
        }
        self.locals.truncate(mark);
        Ok(())
    }

    fn fresh(&mut self) -> String {
        self.next_local += 1;
        format!("v{}", self.next_local)
    }

    fn local_of(&mut self, ty: Ty) -> Result<Option<String>> {
        let found: Vec<&String> = self
            .locals
            .iter()
            .filter(|(_, t)| *t == ty)
            .map(|(n, _)| n)
            .collect();
        if found.is_empty() {
            return Ok(None);
        }
        Ok(Some((*self.u.choose(&found)?).clone()))
    }

    fn stmt(&mut self, depth: u32) -> Result<()> {
        self.fuel = self.fuel.max(30);
        let kind = self.u.int_in_range(0..=17)?;
        match kind {
            0..=3 => {
                let ty = self.any_ty()?;
                let name = self.fresh();
                let form = self.u.int_in_range(0..=2)?;
                let text = match form {
                    0 => format!("{name} := {};", self.expr(ty, 3)?),
                    1 => format!("{name}: {} = {};", self.ty_name(ty), self.expr(ty, 3)?),
                    _ => format!("{name}: {};", self.ty_name(ty)),
                };
                self.line(&text);
                self.locals.push((name, ty));
            }
            4 | 5 => {
                let ty = *self.pick(&[Ty::Int, Ty::S32, Ty::U8, Ty::Float, Ty::Bool, Ty::Str])?;
                if let Some(target) = self.lvalue(ty)? {
                    let op = match ty {
                        Ty::Int | Ty::S32 | Ty::U8 => *self.pick(&[
                            "=", "+=", "-=", "*=", "/=", "%=", "&=", "|=", "^=", "<<=", ">>=",
                        ])?,
                        Ty::Float => *self.pick(&["=", "+=", "-=", "*=", "/="])?,
                        _ => "=",
                    };
                    let v = self.expr(ty, 3)?;
                    self.line(&format!("{target} {op} {v};"));
                }
            }
            6 if depth > 0 => {
                let c = self.expr(Ty::Bool, 3)?;
                self.line(&format!("if {c} {{"));
                self.nested(depth)?;
                if self.chance(1, 2)? {
                    self.line("} else {");
                    self.nested(depth)?;
                }
                self.line("}");
            }
            7 if depth > 0 => {
                let n = self.u.int_in_range(0..=12)?;
                let form = self.u.int_in_range(0..=3)?;
                let var = match form {
                    1 | 3 => self.fresh(),
                    _ => "it".to_string(),
                };
                let head = match form {
                    0 => format!("for 0..{n}"),
                    1 => format!("for {var}: 0..{n}"),
                    2 => format!("for < 0..{n}"),
                    _ => format!("for < {var}: 0..{n}"),
                };
                self.line(&format!("{head} {{"));
                self.locals.push((var, Ty::Int));
                self.loop_depth += 1;
                self.nested(depth)?;
                self.loop_depth -= 1;
                self.locals.pop();
                self.line("}");
            }
            8 if depth > 0 => {
                let arr = [Ty::Array, Ty::View, Ty::Dynamic];
                let ty = *self.pick(&arr)?;
                if let Some(a) = self.local_of(ty)? {
                    let by_pointer = self.chance(1, 3)?;
                    let star = if by_pointer {
                        "*"
                    } else {
                        ""
                    };
                    self.line(&format!("for {star}{a} {{"));
                    self.locals.push(("it_index".into(), Ty::Int));
                    if !by_pointer {
                        self.locals.push(("it".into(), Ty::Int));
                    }
                    self.loop_depth += 1;
                    self.nested(depth)?;
                    if self.chance(1, 5)? && ty == Ty::Dynamic {
                        self.line("    remove it;");
                    }
                    self.loop_depth -= 1;
                    self.locals.pop();
                    if !by_pointer {
                        self.locals.pop();
                    }
                    self.line("}");
                }
            }
            9 if depth > 0 => {
                let counter = self.fresh();
                let limit = self.u.int_in_range(0..=20)?;
                self.line(&format!("{counter} := 0;"));
                self.line(&format!("while {counter} < {limit} {{"));
                self.line(&format!("    {counter} += 1;"));
                self.loop_depth += 1;
                self.nested(depth)?;
                self.loop_depth -= 1;
                self.line("}");
            }
            10 if depth > 0 => {
                // `if x == { case ...; }` over an int or an enum.
                if !self.enums.is_empty() && self.chance(1, 2)? {
                    let e = self.u.choose_index(self.enums.len())?;
                    let v = self.expr(Ty::Enum(e), 2)?;
                    let complete = if self.chance(1, 3)? {
                        " #complete"
                    } else {
                        ""
                    };
                    self.line(&format!("if{complete} {v} == {{"));
                    for m in 0..self.enums[e] {
                        if self.chance(3, 4)? {
                            self.line(&format!("  case .M{m};"));
                            self.nested(depth)?;
                        }
                    }
                } else {
                    let v = self.expr(Ty::Int, 2)?;
                    self.line(&format!("if {v} == {{"));
                    for _ in 0..self.u.int_in_range(0..=3)? {
                        let c = self.u.int_in_range(-2..=4)?;
                        self.line(&format!("  case {c};"));
                        self.nested(depth)?;
                        if self.chance(1, 4)? {
                            self.line("    #through;");
                        }
                    }
                    self.line("  case;");
                }
                self.line("}");
            }
            11 if self.loop_depth > 0 => {
                let word = *self.pick(&["break;", "continue;"])?;
                self.line(word);
            }
            12 => {
                let v = self.expr_any(3)?;
                let text = if self.basic && self.chance(2, 3)? {
                    format!("print(\"%\\n\", {v});")
                } else {
                    format!("{v};")
                };
                self.line(&text);
            }
            13 if depth > 0 => {
                self.line("defer {");
                self.nested(depth)?;
                self.line("}");
            }
            14 if depth > 0 => {
                self.line("{");
                self.nested(depth)?;
                self.line("}");
            }
            15 => {
                if let Some(d) = self.local_of(Ty::Dynamic)?
                    && self.basic
                {
                    let v = self.expr(Ty::Int, 2)?;
                    let call = *self.pick(&[
                        "array_add",
                        "array_add_if_unique",
                        "array_unordered_remove_by_value",
                    ])?;
                    self.line(&format!("{call}(*{d}, {v});"));
                } else if let Some(p) = self.local_of(Ty::Pointer)? {
                    let v = self.expr(Ty::Int, 2)?;
                    self.line(&format!("if {p} {{ {p}.* = {v}; }}"));
                }
            }
            16 => {
                // Macro use: a `Code` argument or a backtick into this scope.
                if self.macros.is_empty() {
                    return Ok(());
                }
                let (m, takes_code) = *self.u.choose(&self.macros)?;
                if takes_code {
                    if let Some(target) = self.lvalue(Ty::Int)? {
                        self.line(&format!("M{m}(#code {target} += 1);"));
                    }
                } else {
                    let result = format!("result_{m}");
                    if !self.locals.iter().any(|(n, _)| *n == result) {
                        self.line(&format!("{result} := 0;"));
                        self.locals.push((result.clone(), Ty::Int));
                    }
                    let v = self.expr(Ty::Int, 2)?;
                    self.line(&format!("{result} += M{m}({v});"));
                }
            }
            17 => {
                if let Some(t) = self.returns {
                    let v = self.expr(t, 2)?;
                    if self.chance(1, 4)? {
                        let c = self.expr(Ty::Bool, 1)?;
                        self.line(&format!("if {c} return {v};"));
                    }
                } else if self.chance(1, 8)? {
                    self.line("return;");
                }
            }
            _ => {
                let v = self.expr_any(2)?;
                self.line(&format!("_ := {v};"));
            }
        }
        Ok(())
    }

    fn nested(&mut self, depth: u32) -> Result<()> {
        self.indent += 1;
        let mark = self.locals.len();
        let count = self.u.int_in_range(0..=3)?;
        for _ in 0..count {
            if self.u.is_empty() {
                break;
            }
            self.stmt(depth - 1)?;
        }
        self.locals.truncate(mark);
        self.indent -= 1;
        Ok(())
    }

    /// An assignable place of type `ty` (a local, a struct field, an array element).
    fn lvalue(&mut self, ty: Ty) -> Result<Option<String>> {
        if let Some(name) = self.local_of(ty)?
            && name != "it"
            && name != "it_index"
            && !name.starts_with('p')
        {
            return Ok(Some(name));
        }
        if ty == Ty::Int
            && let Some(a) = self.local_of(Ty::Array)?
        {
            let i = self.expr(Ty::Int, 1)?;
            return Ok(Some(format!("{a}[{i}]")));
        }
        for s in 0..self.structs.len() {
            if let Some(v) = self.local_of(Ty::Struct(s))?
                && !v.starts_with('p')
            {
                let fields: Vec<String> = self.structs[s]
                    .fields
                    .iter()
                    .filter(|(_, t)| *t == ty)
                    .map(|(n, _)| n.clone())
                    .collect();
                if !fields.is_empty() {
                    let f = self.u.choose(&fields)?.clone();
                    return Ok(Some(format!("{v}.{f}")));
                }
            }
        }
        Ok(None)
    }

    fn expr_any(&mut self, depth: u32) -> Result<String> {
        let ty = self.any_ty()?;
        self.expr(ty, depth)
    }

    fn call(&mut self, ty: Ty, depth: u32) -> Result<String> {
        let candidates: Vec<usize> = (0..self.procs.len())
            .filter(|&i| self.procs[i].ret == Some(ty))
            .collect();
        if candidates.is_empty() {
            return self.leaf(ty);
        }
        let i = *self.u.choose(&candidates)?;
        self.call_proc(i, depth)
    }

    fn call_proc(&mut self, i: usize, depth: u32) -> Result<String> {
        let params = self.procs[i].params.clone();
        let mut args = Vec::new();
        for (k, p) in params.iter().enumerate() {
            let a = self.expr(*p, depth.saturating_sub(1))?;
            // Named arguments sometimes.
            if self.chance(1, 6)? {
                args.push(format!("p{k} = {a}"));
            } else {
                args.push(a);
            }
        }
        Ok(format!("{}({})", self.procs[i].name, args.join(", ")))
    }

    fn int_literal(&mut self) -> Result<String> {
        Ok(match self.u.int_in_range(0..=9)? {
            0 => "0".into(),
            1 => "1".into(),
            2 => "-1".into(),
            3 => format!("{}", self.u.int_in_range(-100..=100)?),
            4 => format!("0x{:x}", self.u.arbitrary::<u32>()?),
            5 => "0x7fff_ffff_ffff_ffff".into(),
            6 => format!("0b{:b}", self.u.arbitrary::<u8>()?),
            7 => format!("{}", self.u.arbitrary::<i64>()?),
            8 => "#char \"a\"".into(),
            _ => format!("{}", self.u.int_in_range(2..=9)?),
        })
    }

    /// A leaf of type `ty`: a literal, a local, or a constant.
    fn leaf(&mut self, ty: Ty) -> Result<String> {
        if self.chance(1, 2)?
            && let Some(name) = self.local_of(ty)?
        {
            return Ok(name);
        }
        Ok(match ty {
            Ty::Int => {
                if !self.constants.is_empty() && self.chance(1, 4)? {
                    self.u.choose(&self.constants)?.clone()
                } else {
                    self.int_literal()?
                }
            }
            Ty::S32 => format!("cast(s32) {}", self.u.int_in_range(-1000..=1000)?),
            Ty::U8 => format!("cast(u8) {}", self.u.arbitrary::<u8>()?),
            Ty::Float => {
                let f = *self.pick(&["0.0", "1.5", "-2.25", "1e300", "0.1", "3.0"])?;
                f.to_string()
            }
            Ty::F32 => format!("cast(float32) {}", self.u.int_in_range(-9..=9)?),
            Ty::Bool => (*self.pick(&["true", "false"])?).to_string(),
            Ty::Str => {
                let s = *self.pick(&[
                    "\"\"",
                    "\"abc\"",
                    "\"tab\\tnl\\n\"",
                    "\"\\x41\\u00e9\"",
                    "\"%\"",
                    "#string END\nraw\nEND",
                ])?;
                s.to_string()
            }
            Ty::Array => "int.[1, 2, 3, 4]".into(),
            Ty::Dynamic => "[..] int.{}".into(),
            Ty::View => {
                if let Some(a) = self.local_of(Ty::Array)? {
                    a
                } else {
                    "int.[5, 6]".into()
                }
            }
            Ty::Struct(i) => format!("S{i}.{{}}"),
            Ty::Enum(i) => format!("E{i}.M{}", self.u.choose_index(self.enums[i])?),
            Ty::Pointer => {
                if let Some(v) = self.local_of(Ty::Int)?
                    && !v.starts_with("it")
                {
                    format!("*{v}")
                } else {
                    "null".into()
                }
            }
            Ty::Boxed => format!("Box(int).{{{}}}", self.int_literal()?),
        })
    }

    fn expr(&mut self, ty: Ty, depth: u32) -> Result<String> {
        if depth == 0 || self.fuel == 0 || self.u.is_empty() {
            return self.leaf(ty);
        }
        self.fuel -= 1;
        // A deliberately ill-typed expression now and then.
        if self.chaos && self.chance(1, 20)? {
            let other = self.any_ty()?;
            return self.leaf(other);
        }
        let d = depth - 1;
        let choice = self.u.int_in_range(0..=9)?;
        if choice < 3 {
            return self.leaf(ty);
        }
        if choice == 3 {
            return self.call(ty, d);
        }
        if choice == 4 {
            let c = self.expr(Ty::Bool, d)?;
            let a = self.expr(ty, d)?;
            let b = self.expr(ty, d)?;
            return Ok(format!("(ifx {c} then {a} else {b})"));
        }
        if choice == 5 && SCALARS.contains(&ty) && ty != Ty::Str {
            // Compile-time code cannot see runtime locals.
            let saved = std::mem::take(&mut self.locals);
            let inner = self.expr(ty, d);
            self.locals = saved;
            return Ok(format!("#run {}", inner?));
        }
        Ok(match ty {
            Ty::Int | Ty::S32 | Ty::U8 => match self.u.int_in_range(0..=9)? {
                0..=3 => {
                    let op = *self.pick(&["+", "-", "*", "/", "%", "&", "|", "^", "<<", ">>"])?;
                    format!("({} {op} {})", self.expr(ty, d)?, self.expr(ty, d)?)
                }
                4 => format!("-{}", self.expr(ty, d)?),
                5 => format!("~{}", self.expr(ty, d)?),
                6 => {
                    let from = *self.pick(&[Ty::Float, Ty::Int, Ty::U8, Ty::Bool, Ty::S32])?;
                    let into = self.ty_name(ty);
                    let trunc = if self.chance(1, 2)? {
                        ",trunc"
                    } else {
                        ""
                    };
                    format!("cast{trunc}({into}) {}", self.expr(from, d)?)
                }
                7 => {
                    let a = *self.pick(&[Ty::Array, Ty::View, Ty::Dynamic, Ty::Str])?;
                    let v = self.expr(a, d)?;
                    if a == Ty::Str || self.chance(1, 2)? {
                        let cast = if ty == Ty::Int {
                            ""
                        } else {
                            "xx "
                        };
                        format!("{cast}({v}).count")
                    } else {
                        let i = self.expr(Ty::Int, d)?;
                        let cast = if ty == Ty::Int {
                            ""
                        } else {
                            "xx "
                        };
                        format!("{cast}{v}[{i}]")
                    }
                }
                8 => {
                    let t = self.any_ty()?;
                    let name = self.ty_name(t);
                    let cast = if ty == Ty::Int {
                        ""
                    } else {
                        "xx "
                    };
                    format!("{cast}size_of({name})")
                }
                _ => {
                    if let Some(f) = self.field_of(ty, d)? {
                        f
                    } else if !self.enums.is_empty() && ty == Ty::Int {
                        let e = self.u.choose_index(self.enums.len())?;
                        format!("cast(int) {}", self.expr(Ty::Enum(e), d)?)
                    } else {
                        let b = self.expr(Ty::Boxed, d)?;
                        let cast = if ty == Ty::Int {
                            ""
                        } else {
                            "xx "
                        };
                        format!("{cast}{b}.v")
                    }
                }
            },
            Ty::Float | Ty::F32 => match self.u.int_in_range(0..=4)? {
                0..=2 => {
                    let op = *self.pick(&["+", "-", "*", "/"])?;
                    format!("({} {op} {})", self.expr(ty, d)?, self.expr(ty, d)?)
                }
                3 => format!("cast({}) {}", self.ty_name(ty), self.expr(Ty::Int, d)?),
                _ => self.field_of(ty, d)?.map_or_else(|| self.leaf(ty), Ok)?,
            },
            Ty::Bool => match self.u.int_in_range(0..=5)? {
                0..=2 => {
                    let t = *self.pick(&[Ty::Int, Ty::Float, Ty::Str, Ty::U8, Ty::Bool])?;
                    let op = if matches!(t, Ty::Str | Ty::Bool) {
                        *self.pick(&["==", "!="])?
                    } else {
                        *self.pick(&["==", "!=", "<", "<=", ">", ">="])?
                    };
                    format!("({} {op} {})", self.expr(t, d)?, self.expr(t, d)?)
                }
                3 => {
                    let op = *self.pick(&["&&", "||"])?;
                    format!(
                        "({} {op} {})",
                        self.expr(Ty::Bool, d)?,
                        self.expr(Ty::Bool, d)?
                    )
                }
                4 => format!("!{}", self.expr(Ty::Bool, d)?),
                _ => {
                    if !self.enums.is_empty() {
                        let e = self.u.choose_index(self.enums.len())?;
                        format!(
                            "({} == {})",
                            self.expr(Ty::Enum(e), d)?,
                            self.leaf(Ty::Enum(e))?
                        )
                    } else {
                        self.leaf(ty)?
                    }
                }
            },
            Ty::Str => match self.u.int_in_range(0..=2)? {
                0 if self.basic => {
                    format!(
                        "tprint(\"%-%\", {}, {})",
                        self.expr_any(d)?,
                        self.expr_any(d)?
                    )
                }
                1 => self.field_of(ty, d)?.map_or_else(|| self.leaf(ty), Ok)?,
                _ => self.leaf(ty)?,
            },
            Ty::Struct(i) => {
                let fields = self.structs[i].fields.clone();
                let mut parts = Vec::new();
                for (name, t) in fields.iter().take(3) {
                    if SCALARS.contains(t) && self.chance(1, 2)? {
                        parts.push(format!("{name} = {}", self.expr(*t, d)?));
                    }
                }
                format!("S{i}.{{{}}}", parts.join(", "))
            }
            Ty::Enum(i) => {
                if self.chance(1, 2)? {
                    format!("E{i}.M{}", self.u.choose_index(self.enums[i])?)
                } else {
                    format!("cast(E{i}) {}", self.u.int_in_range(0..=6)?)
                }
            }
            Ty::Array => {
                let mut items = Vec::new();
                for _ in 0..4 {
                    items.push(self.expr(Ty::Int, d)?);
                }
                format!("int.[{}]", items.join(", "))
            }
            _ => self.leaf(ty)?,
        })
    }

    /// `local.field` of type `ty`, if some struct local has such a field.
    fn field_of(&mut self, ty: Ty, _depth: u32) -> Result<Option<String>> {
        for s in 0..self.structs.len() {
            let fields: Vec<String> = self.structs[s]
                .fields
                .iter()
                .filter(|(_, t)| *t == ty)
                .map(|(n, _)| n.clone())
                .collect();
            if fields.is_empty() {
                continue;
            }
            if let Some(v) = self.local_of(Ty::Struct(s))? {
                let f = self.u.choose(&fields)?.clone();
                return Ok(Some(format!("{v}.{f}")));
            }
        }
        Ok(None)
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn renders_main_even_from_empty_input() {
        let text = super::program(&[]);
        assert!(text.contains("main :: () {"), "{text}");
    }

    #[test]
    fn varied_inputs_render_varied_programs() {
        let a = super::program(&[7; 300]);
        let b = super::program(&(0..=255).collect::<Vec<u8>>());
        assert_ne!(a, b);
    }
}
