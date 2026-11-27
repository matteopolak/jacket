//! `#[derive(Args)]`: see the `argdef` crate for what the attributes mean.
use proc_macro::TokenStream;
use proc_macro2::TokenStream as Tokens;
use quote::{ToTokens, quote};
use syn::{
    Attribute, DeriveInput, Expr, ExprLit, Field, Lit, LitStr, Meta, Path, Type, parse_macro_input,
};

/// Derive `argdef::Args` for a struct with named fields.
#[proc_macro_derive(Args, attributes(args, arg))]
pub fn derive_args(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    expand(&input)
        .unwrap_or_else(|error| error.to_compile_error())
        .into()
}

/// The lines of the `///` comments in `attrs`, each without its leading space.
fn doc_lines(attrs: &[Attribute]) -> Vec<String> {
    let mut lines = Vec::new();
    for attr in attrs {
        if let Meta::NameValue(pair) = &attr.meta
            && pair.path.is_ident("doc")
            && let Expr::Lit(ExprLit {
                lit: Lit::Str(text),
                ..
            }) = &pair.value
        {
            let line = text.value();
            lines.push(
                line.strip_prefix(' ')
                    .unwrap_or(&line)
                    .trim_end()
                    .to_string(),
            );
        }
    }
    lines
}

/// What the type of a field makes of an option.
enum Shape {
    Bool,
    Option(Type),
    Vec(Type),
    Plain,
}

fn shape(ty: &Type) -> Shape {
    if let Type::Path(path) = ty
        && let Some(last) = path.path.segments.last()
    {
        if last.ident == "bool" {
            return Shape::Bool;
        }
        if (last.ident == "Option" || last.ident == "Vec")
            && let syn::PathArguments::AngleBracketed(args) = &last.arguments
            && let Some(syn::GenericArgument::Type(inner)) = args.args.first()
        {
            return if last.ident == "Option" {
                Shape::Option(inner.clone())
            } else {
                Shape::Vec(inner.clone())
            };
        }
    }
    Shape::Plain
}

/// One `#[arg(...)]`.
#[derive(Default)]
struct Arg {
    names: Vec<LitStr>,
    aliases: Vec<LitStr>,
    value: Option<LitStr>,
    needs: Option<LitStr>,
    synopsis: Option<LitStr>,
    help: Option<LitStr>,
    section: Option<LitStr>,
    hint: Option<LitStr>,
    tag: Option<LitStr>,
    until: Option<LitStr>,
    rest: Option<LitStr>,
    group: Option<LitStr>,
    conflict: Option<Expr>,
    parse: Option<Path>,
    with: Option<Path>,
    set: Option<Expr>,
    positional: bool,
    flatten: bool,
    hidden: bool,
    exit: bool,
    attached: bool,
    skip: bool,
    os: bool,
}

fn parse_arg(attr: &Attribute) -> syn::Result<Arg> {
    let mut arg = Arg::default();
    attr.parse_nested_meta(|meta| {
        let key = meta
            .path
            .get_ident()
            .map(ToString::to_string)
            .unwrap_or_default();
        match key.as_str() {
            "positional" => arg.positional = true,
            "flatten" => arg.flatten = true,
            "hidden" => arg.hidden = true,
            "exit" => arg.exit = true,
            "attached" => arg.attached = true,
            "skip" => arg.skip = true,
            "os" => arg.os = true,
            "conflict" => arg.conflict = Some(meta.value()?.parse()?),
            "name" => arg.names.push(meta.value()?.parse()?),
            "alias" => arg.aliases.push(meta.value()?.parse()?),
            "value" => arg.value = Some(meta.value()?.parse()?),
            "needs" => arg.needs = Some(meta.value()?.parse()?),
            "synopsis" => arg.synopsis = Some(meta.value()?.parse()?),
            "help" => arg.help = Some(meta.value()?.parse()?),
            "section" => arg.section = Some(meta.value()?.parse()?),
            "hint" => arg.hint = Some(meta.value()?.parse()?),
            "tag" => arg.tag = Some(meta.value()?.parse()?),
            "until" => arg.until = Some(meta.value()?.parse()?),
            "rest" => arg.rest = Some(meta.value()?.parse()?),
            "group" => arg.group = Some(meta.value()?.parse()?),
            "parse" => arg.parse = Some(meta.value()?.parse()?),
            "with" => arg.with = Some(meta.value()?.parse()?),
            "set" => arg.set = Some(meta.value()?.parse()?),
            _ => return Err(meta.error(format!("unknown `arg` key `{key}`"))),
        }
        Ok(())
    })?;
    Ok(arg)
}

fn opt_str(value: Option<&LitStr>) -> Tokens {
    value.map_or_else(|| quote!(""), |text| quote!(#text))
}

fn strs(values: &[LitStr]) -> Tokens {
    quote!(&[#(#values),*])
}

/// The `Opt` literal and `apply` arm of one `#[arg]`.
fn option(
    field: &Field,
    arg: &Arg,
    section: Option<&LitStr>,
    is_first: bool,
    doc: &str,
    id: usize,
) -> syn::Result<(Tokens, Tokens)> {
    let ident = field.ident.as_ref().expect("named field");
    let shape = shape(&field.ty);
    let (kind, arm) = if arg.positional {
        (
            quote!(Positional),
            quote!(::argdef::Input::Arg(a) => self.#ident.push(a.to_string()),),
        )
    } else if arg.rest.is_some() {
        (
            quote!(Rest),
            quote!(
                ::argdef::Input::Begin => {
                    self.#ident.get_or_insert_with(Vec::new);
                }
                ::argdef::Input::Arg(a) => self.#ident.get_or_insert_with(Vec::new).push(a.to_string()),
            ),
        )
    } else if arg.group.is_some() {
        (
            quote!(Group),
            quote!(
                ::argdef::Input::Start(n) => self.#ident.push(::argdef::Group::start(n)),
                ::argdef::Input::Arg(a) => {
                    if let Some(group) = self.#ident.last_mut() {
                        ::argdef::Group::push(group, a);
                    }
                }
            ),
        )
    } else if let Some(set) = &arg.set {
        let store = match &shape {
            Shape::Option(_) => {
                if let Some(message) = &arg.conflict {
                    quote!({
                        let new = #set;
                        if let Some(old) = &self.#ident
                            && *old != new
                        {
                            return Err(::argdef::Error::invalid(#message));
                        }
                        self.#ident = Some(new);
                    })
                } else {
                    quote!(self.#ident = Some(#set))
                }
            }
            Shape::Vec(_) => quote!(self.#ident.push(#set)),
            _ => quote!(self.#ident = #set),
        };
        (quote!(Flag), quote!(::argdef::Input::Flag => { #store; }))
    } else if matches!(shape, Shape::Bool) && arg.with.is_none() {
        (
            quote!(Flag),
            quote!(::argdef::Input::Flag => { self.#ident = true; }),
        )
    } else {
        let store = if let Some(with) = &arg.with {
            quote!(#with(self, name, v)?;)
        } else {
            let (inner, wrap): (&Type, fn(&Tokens, &syn::Ident) -> Tokens) = match &shape {
                Shape::Option(t) => (t, |p, f| quote!(self.#f = Some(#p);)),
                Shape::Vec(t) => (t, |p, f| quote!(self.#f.push(#p);)),
                Shape::Plain | Shape::Bool => (&field.ty, |p, f| quote!(self.#f = #p;)),
            };
            let default = if arg.os {
                quote!(::argdef::FromOsArg::from_os_arg)
            } else {
                quote!(::argdef::FromArg::from_arg)
            };
            let parse = arg
                .parse
                .as_ref()
                .map_or_else(|| default, |path| quote!(#path));
            let call = quote!(#parse(name, v)?);
            let typed = quote!({ let parsed: #inner = #call; parsed });
            wrap(&typed, ident)
        };
        if arg.os {
            (quote!(Value), quote!(::argdef::Input::Os(v) => { #store }))
        } else {
            (
                quote!(Value),
                quote!(::argdef::Input::Value(v) => { #store }),
            )
        }
    };
    let names = if let Some(rest) = &arg.rest {
        strs(std::slice::from_ref(rest))
    } else {
        strs(&arg.names)
    };
    let aliases = strs(&arg.aliases);
    let prefix = opt_str(arg.group.as_ref());
    let value = match (&arg.value, &kind.to_string()[..]) {
        (Some(v), _) => quote!(#v),
        (None, "Value") => quote!("value"),
        _ => quote!(""),
    };
    let help = match (&arg.help, is_first) {
        (Some(h), _) => quote!(#h),
        (None, true) => quote!(#doc),
        (None, false) => quote!(""),
    };
    let section = opt_str(arg.section.as_ref().or(section));
    let (needs, synopsis, hint, tag, until) = (
        opt_str(arg.needs.as_ref()),
        opt_str(arg.synopsis.as_ref()),
        opt_str(arg.hint.as_ref()),
        opt_str(arg.tag.as_ref()),
        opt_str(arg.until.as_ref()),
    );
    let (hidden, exit, attached, os) = (arg.hidden, arg.exit, arg.attached, arg.os);
    let literal = quote!(::argdef::Opt {
        id: offset + #id,
        kind: ::argdef::Kind::#kind,
        names: #names,
        aliases: #aliases,
        value: #value,
        needs: #needs,
        synopsis: #synopsis,
        help: #help,
        section: #section,
        hint: #hint,
        tag: #tag,
        hidden: #hidden,
        exit: #exit,
        attached: #attached,
        os: #os,
        until: #until,
        prefix: #prefix,
    });
    let arm = quote!(#id => match input { #arm _ => {} },);
    Ok((literal, arm))
}

/// Whether `ty` is `Vec<String>`.
fn is_string_vec(ty: &Type) -> bool {
    matches!(shape(ty), Shape::Vec(Type::Path(inner))
        if inner.path.segments.last().is_some_and(|s| s.ident == "String"))
}

/// Reject an `#[arg]` that cannot mean anything, with the error on the attribute.
fn check(
    attr: &Attribute,
    arg: &Arg,
    field: &Field,
    seen: &mut std::collections::BTreeSet<String>,
) -> syn::Result<()> {
    let wrong = |message: &str| Err(syn::Error::new_spanned(attr, message));
    if arg.flatten || arg.skip {
        return Ok(());
    }
    if arg.positional {
        if !is_string_vec(&field.ty) || !arg.names.is_empty() {
            return wrong("a `positional` field is a `Vec<String>` with no `name`");
        }
        return Ok(());
    }
    if arg.rest.is_some() {
        let ok = matches!(shape(&field.ty), Shape::Option(inner) if is_string_vec(&inner));
        return if ok {
            Ok(())
        } else {
            wrong("a `rest` field is an `Option<Vec<String>>`")
        };
    }
    if arg.group.is_some() {
        return if matches!(shape(&field.ty), Shape::Vec(_)) {
            Ok(())
        } else {
            wrong("a `group` field is a `Vec` of a type that implements `argdef::Group`")
        };
    }
    if arg.names.is_empty() {
        return wrong("an option needs a `name = \"...\"`");
    }
    if arg.conflict.is_some()
        && !(arg.set.is_some() && matches!(shape(&field.ty), Shape::Option(_)))
    {
        return wrong("`conflict` only applies to a `set` option on an `Option` field");
    }
    for spelling in arg.names.iter().chain(&arg.aliases) {
        if !seen.insert(spelling.value()) {
            return Err(syn::Error::new_spanned(
                spelling,
                format!("`{}` is spelt by more than one option", spelling.value()),
            ));
        }
    }
    Ok(())
}

fn expand(input: &DeriveInput) -> syn::Result<Tokens> {
    let name = &input.ident;
    if !input.generics.params.is_empty() {
        return Err(syn::Error::new_spanned(
            &input.generics,
            "Args does not derive on generic structs",
        ));
    }
    let syn::Data::Struct(data) = &input.data else {
        return Err(syn::Error::new_spanned(input, "Args derives on structs"));
    };
    let syn::Fields::Named(fields) = &data.fields else {
        return Err(syn::Error::new_spanned(input, "Args needs named fields"));
    };
    // Container attributes.
    let mut meta_name = None;
    let mut version: Option<Expr> = None;
    let mut usage = None;
    let mut sections: Vec<LitStr> = Vec::new();
    for attr in input.attrs.iter().filter(|a| a.path().is_ident("args")) {
        attr.parse_nested_meta(|meta| {
            if meta.path.is_ident("name") {
                meta_name = Some(meta.value()?.parse::<LitStr>()?);
            } else if meta.path.is_ident("version") {
                version = Some(meta.value()?.parse()?);
            } else if meta.path.is_ident("usage") {
                usage = Some(meta.value()?.parse::<LitStr>()?);
            } else if meta.path.is_ident("sections") {
                let content;
                let value = meta.value()?;
                syn::bracketed!(content in value);
                let list = content
                    .parse_terminated(<LitStr as syn::parse::Parse>::parse, syn::Token![,])?;
                sections.extend(list);
            } else {
                let key = meta
                    .path
                    .get_ident()
                    .map(ToString::to_string)
                    .unwrap_or_default();
                return Err(meta.error(format!(
                    "unknown `args` key `{key}` (the keys are name, version, usage and sections)"
                )));
            }
            Ok(())
        })?;
    }
    let docs = doc_lines(&input.attrs);
    let about = docs.first().cloned().unwrap_or_default();
    let description = docs
        .get(1..)
        .unwrap_or_default()
        .join("\n")
        .trim()
        .to_string();
    let meta_name = opt_str(meta_name.as_ref());
    let version = version.map_or_else(|| quote!(""), |v| v.to_token_stream());
    let usage = opt_str(usage.as_ref());

    let mut options = Vec::new();
    let mut arms = Vec::new();
    let mut flattened: Vec<(&syn::Ident, &Type)> = Vec::new();
    let mut last_section: Option<LitStr> = None;
    let mut seen = std::collections::BTreeSet::new();
    for field in &fields.named {
        let ident = field.ident.as_ref().expect("named");
        let mut args = Vec::new();
        for attr in field.attrs.iter().filter(|a| a.path().is_ident("arg")) {
            let arg = parse_arg(attr)?;
            check(attr, &arg, field, &mut seen)?;
            args.push(arg);
        }
        if args.iter().any(|a| a.flatten) {
            flattened.push((ident, &field.ty));
            continue;
        }
        if args.iter().any(|a| a.skip) {
            continue;
        }
        let doc = doc_lines(&field.attrs).join("\n");
        let Some(first) = args.first() else {
            return Err(syn::Error::new_spanned(
                field,
                "a field needs `#[arg(...)]`",
            ));
        };
        // A field without a section is listed under the one before it.
        let field_section = first.section.clone().or_else(|| last_section.clone());
        last_section.clone_from(&field_section);
        for (index, arg) in args.iter().enumerate() {
            let id = options.len();
            let (literal, arm) = option(field, arg, field_section.as_ref(), index == 0, &doc, id)?;
            options.push(literal);
            arms.push(arm);
        }
    }
    let own = options.len();
    let child_types: Vec<&Type> = flattened.iter().map(|(_, ty)| *ty).collect();
    let mut child_options = Vec::new();
    let mut child_dispatch = Vec::new();
    for (ident, ty) in &flattened {
        child_options.push(quote!(
            <#ty as ::argdef::Args>::options(offset + base, out);
            base += <#ty as ::argdef::Args>::COUNT;
        ));
        child_dispatch.push(quote!(
            if rel < <#ty as ::argdef::Args>::COUNT {
                return ::argdef::Args::apply(&mut self.#ident, rel, name, input);
            }
            rel -= <#ty as ::argdef::Args>::COUNT;
        ));
    }
    Ok(quote!(
        impl ::argdef::Args for #name {
            const COUNT: usize = #own #(+ <#child_types as ::argdef::Args>::COUNT)*;
            const META: ::argdef::Meta = ::argdef::Meta {
                name: #meta_name,
                version: #version,
                usage: #usage,
                about: #about,
                description: #description,
                sections: &[#(#sections),*],
            };

            fn options(offset: usize, out: &mut Vec<::argdef::Opt>) {
                out.extend([#(#options),*]);
                let mut base = #own;
                #(#child_options)*
                let _ = base;
            }

            fn apply(
                &mut self,
                id: usize,
                name: &str,
                input: ::argdef::Input<'_>,
            ) -> Result<(), ::argdef::Error> {
                match id {
                    #(#arms)*
                    _ => {
                        let mut rel = id - #own;
                        #(#child_dispatch)*
                        let _ = rel;
                    }
                }
                Ok(())
            }
        }
    ))
}

#[cfg(test)]
mod tests {
    use super::expand;

    /// The message `#[derive(Args)]` gives for `source`, if it refuses it.
    fn refused(source: &str) -> Option<String> {
        let input = syn::parse_str(source).expect("test source parses");
        expand(&input).err().map(|e| e.to_string())
    }

    #[test]
    fn a_good_struct_is_accepted() {
        let source = r#"struct A {
            #[arg(name = "-a", name = "-b", alias = "-c")] a: bool,
            #[arg(positional)] files: Vec<String>,
            #[arg(rest = "-", until = "--")] rest: Option<Vec<String>>,
            #[arg(name = "-o", set = 1, conflict = "x")] o: Option<u8>,
        }"#;
        assert_eq!(refused(source), None);
    }

    #[test]
    fn mistakes_are_errors_that_say_what_is_wrong() {
        let cases = [
            (
                "struct A { #[arg(needs = \"a file\")] a: bool }",
                "needs a `name",
            ),
            (
                "struct A { #[arg(name = \"-a\", conflict = \"x\")] a: bool }",
                "`conflict` only applies",
            ),
            (
                "struct A { #[arg(name = \"-a\", set = 1, conflict = \"x\")] a: u8 }",
                "`conflict` only applies",
            ),
            (
                "struct A { #[arg(name = \"-a\")] a: bool, #[arg(name = \"-b\", alias = \"-a\")] b: bool }",
                "`-a` is spelt by more than one",
            ),
            (
                "struct A<T> { #[arg(name = \"-a\")] a: T }",
                "generic structs",
            ),
            ("struct A { #[arg(positional)] a: String }", "`Vec<String>`"),
            (
                "struct A { #[arg(positional)] a: Vec<u8> }",
                "`Vec<String>`",
            ),
            (
                "struct A { #[arg(rest = \"-\")] a: Vec<String> }",
                "`Option<Vec<String>>`",
            ),
            ("struct A { #[arg(group = \"+\")] a: bool }", "`Vec`"),
            (
                "#[args(colour = \"x\")] struct A { #[arg(name = \"-a\")] a: bool }",
                "unknown `args` key `colour`",
            ),
            ("struct A { a: bool }", "a field needs"),
        ];
        for (source, expected) in cases {
            let message = refused(source).unwrap_or_else(|| panic!("accepted: {source}"));
            assert!(message.contains(expected), "{source}: {message}");
        }
    }
}
