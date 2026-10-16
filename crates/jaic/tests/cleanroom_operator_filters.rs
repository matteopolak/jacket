//! Public filter symbols select original operator declarations, never their internal names.
use jaic::sema::{Compiler, Options, VirtualFs};
use std::{path::Path, rc::Rc};

const OPERATORS: &str = r#"
Value :: struct { n: int; }
operator + :: (a:Value,b:Value)->Value { return .{a.n+b.n}; }
operator - :: (a:Value,b:Value)->Value { return .{a.n-b.n}; }
operator - :: (a:Value)->Value { return .{-a.n}; }
operator * :: (a:Value,b:Value)->Value { return .{a.n*b.n}; }
operator / :: (a:Value,b:Value)->Value { return .{a.n/b.n}; }
operator == :: (a:Value,b:Value)->bool { return a.n%10==b.n%10; }
operator != :: (a:Value,b:Value)->bool { return a.n%10!=b.n%10; }
operator < :: (a:Value,b:Value)->bool { return a.n<b.n; }
operator <= :: (a:Value,b:Value)->bool { return a.n<=b.n; }
operator > :: (a:Value,b:Value)->bool { return a.n>b.n; }
operator >= :: (a:Value,b:Value)->bool { return a.n>=b.n; }
Index :: struct { data:[3]int; }
Hidden :: struct { n:int; }
operator [] :: (a:Index,i:int)->int { return a.data[i]; }
operator []= :: (a:*Index,i:int,value:int) { a.data[i]=value; }
operator *[] :: (a:*Index,i:int)->*int { return *a.data[i]; }
operator_count :: 37;
storage: int = 5;
#scope_file
secret :: 91;
Private :: struct { n:int; }
operator + :: (a:Private,b:Private)->Private { return .{a.n+b.n}; }
operator - :: (a:Hidden,b:Hidden)->Hidden { return .{a.n-b.n}; }
"#;

fn compile(main: &str, facade: Option<&str>) -> Result<(), String> {
    let mut fs = VirtualFs::default();
    fs.insert("/lib/Operators/module.jai", OPERATORS.as_bytes());
    if let Some(facade) = facade {
        fs.insert("/lib/Facade/module.jai", facade.as_bytes());
    }
    fs.insert("/main.jai", main.as_bytes());
    let mut options = Options::host();
    options.runtime_support = false;
    options.import_paths = vec!["/lib".into()];
    let mut compiler = Compiler::new(options, Rc::new(fs));
    compiler
        .compile_program(Path::new("/main.jai"))
        .map_err(|error| error.render(&compiler.sources))
}

#[test]
fn reviewed_public_symbols_select_each_operator_family() {
    for (symbol, body) in [
        ("+", "return (a+b).n==15;"),
        ("-", "return (a-b).n==9 && (-b).n == -3;"),
        ("*", "return (a*b).n==36;"),
        ("/", "return (a/b).n==4;"),
        ("==", "return a==M.Value.{2};"),
        ("!=", "return !(a!=M.Value.{2});"),
        ("<", "return b<a;"),
        ("<=", "return b<=a && a<=a;"),
        (">", "return a>b;"),
        (">=", "return a>=b && a>=a;"),
        ("[]", "v:M.Index; v.data[1]=8; return v[1]==8;"),
        ("[]=", "v:M.Index; v[1]=9; return v.data[1]==9;"),
        ("*[]", "v:M.Index; v.data[1]=11; return v[1]==11;"),
    ] {
        let source = format!(
            r#"M :: #import "Operators"; using,only(.[{symbol:?}]) M;
            check :: ()->bool {{ a:=M.Value.{{12}}; b:=M.Value.{{3}}; {body} }}
            #assert check(); main :: () {{}}"#
        );
        compile(&source, None).unwrap_or_else(|error| panic!("{symbol}: {error}"));
    }
}

#[test]
fn internal_spelling_is_not_a_public_filter_alias() {
    for filter in [r#".["operator-"]"#, r#".["operator+", "Value"]"#] {
        let source = format!(
            r#"M :: #import "Operators"; using,only({filter}) M;
            main :: () {{ a:=M.Value.{{7}}; b:=M.Value.{{2}}; v:=a-b; }}"#
        );
        compile(&source, None).expect_err("internal spelling must not select minus");
    }
    // An unknown exclusion does not hide the actual public operator symbol.
    compile(
        r#"using,except(.["operator-"]) M :: #import "Operators";
        #assert (Value.{7}-Value.{2}).n==5; main :: () {}"#,
        None,
    )
    .unwrap();
}

#[test]
fn excludes_and_unselected_overloads_stay_hidden() {
    for filter in [r#"only(.["+"])"#, r#"except(.["-"])"#] {
        let source = format!(
            r#"using,{filter} M :: #import "Operators";
            main :: () {{ a:=M.Value.{{7}}; b:=M.Value.{{2}}; v:=a-b; }}"#
        );
        compile(&source, None).expect_err("minus is not exposed by this filter");
    }
}

#[test]
fn each_reviewed_symbol_can_be_excluded_through_a_filtered_reexport() {
    for (symbol, body) in [
        ("+", "return (a+b).n==15;"),
        ("-", "return (a-b).n==9;"),
        ("*", "return (a*b).n==36;"),
        ("/", "return (a/b).n==4;"),
        // Built-in structural equality remains legal after excluding overloads;
        // a named call tests declaration visibility without assuming its absence.
        ("==", "return operator ==(a,a);"),
        ("!=", "return operator !=(a,b);"),
        ("<", "return b<a;"),
        ("<=", "return b<=a;"),
        (">", "return a>b;"),
        (">=", "return a>=b;"),
        ("[]", "v:M.Index; return v[1]==8;"),
        ("[]=", "v:M.Index; v[1]=9; return v.data[1]==9;"),
        ("*[]", "v:M.Index; return v[1]==11;"),
    ] {
        // Expose just this operator first so alternate index overloads cannot
        // legitimately satisfy the expression after the selected one is excluded.
        let facade = format!(r#"using,only(.[{symbol:?}]) #import "Operators";"#);
        let source = format!(
            r#"M :: #import "Operators"; using,except(.[{symbol:?}]) #import "Facade";
            check :: ()->bool {{ a:=M.Value.{{12}}; b:=M.Value.{{3}}; {body} }}
            main :: () {{}}"#
        );
        assert!(
            compile(&source, Some(&facade)).is_err(),
            "excluded {symbol} must stay hidden"
        );
    }
}

#[test]
fn named_and_anonymous_imports_and_filtered_reexports_preserve_identity() {
    for source in [
        r#"using,only(.["-","Value"]) M :: #import "Operators";
            #assert (Value.{7}-M.Value.{2}).n==5; main :: () {}"#,
        r#"using,only(.["-","Value"]) #import "Operators";
            #assert (Value.{7}-Value.{2}).n==5; main :: () {}"#,
    ] {
        compile(source, None).unwrap();
    }
    compile(
        r#"using #import "Facade";
        #assert (Value.{7}-Value.{2}).n==5; main :: () {}"#,
        Some(r#"using,only(.["-","Value"]) #import "Operators";"#),
    )
    .unwrap();
    // Filtering an ordinary alias must not be confused with an operator identity.
    compile(
        r#"using,only(.["subtract","Value"]) #import "Facade";
        #assert subtract(Value.{7},Value.{2}).n==5; main :: () {}"#,
        Some(r#"Value :: M.Value; subtract :: M.operator-; #scope_file M :: #import "Operators";"#),
    )
    .unwrap();
}

#[test]
fn filters_do_not_bypass_privacy_or_rewrite_ordinary_member_names() {
    for target in ["secret", "Private"] {
        let source = format!(
            r#"using,only(.[{target:?}]) #import "Operators";
            leaked :: {target}; main :: () {{}}"#
        );
        compile(&source, None).expect_err("a filter cannot expose private members");
    }
    for operation in ["a-b", "operator -(a,b)"] {
        let source = format!(
            r#"M :: #import "Operators"; using,only(.["-"]) M;
            main :: () {{ a:=M.Hidden.{{7}}; b:=M.Hidden.{{2}}; leaked:={operation}; }}"#
        );
        compile(&source, None)
            .expect_err("a symbol filter cannot expose a private overload of a public type");
    }
    compile(
        r#"using,only(.["operator_count","storage"]) M :: #import "Operators";
        #assert operator_count==37;
        #run { storage=19; }
        #assert M.storage==19; main :: () {}"#,
        None,
    )
    .unwrap();
    compile(
        r#"using,only(.["-"]) M :: #import "Operators";
        unknown :: operator_count; main :: () {}"#,
        None,
    )
    .expect_err("named members remain excluded");
}
