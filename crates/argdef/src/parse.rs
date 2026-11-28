//! Storing a command line into a struct.
use crate::error::Error;
use crate::spec::{Args, Input, Kind, Spec};
use std::ffi::{OsStr, OsString};

/// Where the arguments being read go.
#[derive(Clone, Copy)]
enum Context {
    Options,
    Group(usize),
    Rest(usize),
}

/// The refusal for an argument that is not UTF-8, naming the options whose values may be.
fn not_utf8(spec: &Spec, arg: &OsStr) -> Error {
    let names: Vec<&str> = spec
        .opts
        .iter()
        .filter(|o| o.kind == Kind::Value && o.os)
        .map(|o| o.names[0])
        .collect();
    Error::not_utf8(arg, &names)
}

/// `-name=value` where `name` is an `os` option and the value is not UTF-8.
fn os_inline<'s>(spec: &'s Spec, arg: &OsStr) -> Option<(&'s str, OsString)> {
    let (name, value) = jacket_config::os_text::split_once(arg, b'=')?;
    let opt = spec
        .find(name.to_str()?)
        .filter(|o| o.kind == Kind::Value && o.os)?;
    let spelling = opt.names.iter().chain(opt.aliases).find(|n| name == **n)?;

    Some((spelling, value))
}

pub(crate) fn parse<T: Args, S: AsRef<OsStr>>(spec: &Spec, args: &[S]) -> Result<T, Error> {
    let mut out = T::default();
    let mut context = Context::Options;
    let mut rest = args.iter().map(AsRef::as_ref);

    while let Some(os) = rest.next() {
        let Some(arg) = os.to_str() else {
            if matches!(context, Context::Options)
                && let Some((name, value)) = os_inline(spec, os)
            {
                let id = spec.find(name).expect("found by `os_inline`").id;

                out.apply(id, name, Input::Os(&value))?;

                continue;
            }

            return Err(not_utf8(spec, os));
        };

        if let Context::Rest(id) = context {
            let until = spec.opts[id].until;

            if until.is_empty() || arg != until {
                out.apply(id, arg, Input::Arg(arg))?;
                continue;
            }
        }

        if let Some(marker) = spec.marker(arg) {
            if marker.kind == Kind::Rest {
                context = Context::Rest(marker.id);
                out.apply(marker.id, arg, Input::Begin)?;
            } else {
                context = Context::Group(marker.id);
                out.apply(marker.id, arg, Input::Start(&arg[marker.prefix.len()..]))?;
            }

            continue;
        }

        if let Context::Group(id) = context {
            out.apply(id, arg, Input::Arg(arg))?;
            continue;
        }

        let (name, inline, opt) = spec.split(arg);
        let Some(opt) = opt else {
            if arg.starts_with('-') {
                return Err(Error::unknown(arg));
            }

            let Some(positional) = spec.opts.iter().find(|o| o.kind == Kind::Positional) else {
                return Err(Error::invalid(format!("unexpected argument `{arg}`")));
            };

            out.apply(positional.id, arg, Input::Arg(arg))?;

            continue;
        };

        if opt.kind == Kind::Value {
            let missing = || Error::missing(name, opt.needs);

            if opt.os {
                let value = match inline {
                    Some(value) => OsString::from(value),
                    None => rest.next().ok_or_else(missing)?.to_os_string(),
                };

                out.apply(opt.id, name, Input::Os(&value))?;
            } else {
                let value = if let Some(value) = inline {
                    value
                } else {
                    let next = rest.next().ok_or_else(missing)?;
                    let text = next.to_str().ok_or_else(|| not_utf8(spec, next))?;

                    text.to_string()
                };

                out.apply(opt.id, name, Input::Value(&value))?;
            }
        } else {
            out.apply(opt.id, name, Input::Flag)?;
        }

        if opt.exit {
            break;
        }
    }

    Ok(out)
}
