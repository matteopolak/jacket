use super::{
    Compiler, Exit, Expr, HandleEffect, NativeKind, ProcId, ProcTarget, Relation, Slot, Token,
    TypeId, Validity,
};

/// Authenticate the selected foreign symbol, system library and C ABI shape together.
pub(super) fn native(c: &Compiler, id: ProcId) -> Option<Relation> {
    let p = &c.pr.procs[id.0 as usize];
    let ProcTarget::Foreign(fid) = p.target? else {
        return None;
    };
    let foreign = c.out.program.foreigns.get(fid.0 as usize)?;
    let lib = c.out.program.libraries.get(foreign.library?)?;
    let sig = p.sig.as_ref()?;
    if !lib.system || foreign.is_data || !sig.c_call || sig.returns.len() != 1 {
        return None;
    }
    // Signature.params includes C-ellipsis metadata, not just fixed ABI arguments.
    // Only open has a supported variadic protocol: path, flags, then mode values.
    let open_tail = foreign.symbol == "open"
        && sig.c_varargs
        && sig.params.len() == 3
        && sig.params[..2].iter().all(|p| !p.variadic)
        && sig.params[2].variadic;
    if (sig.c_varargs || sig.params.iter().any(|p| p.variadic)) && !open_tail {
        return None;
    }
    // This first native slice authenticates only the compiler's supported 64-bit ABIs.
    if !matches!(
        c.options.cpu,
        jaic::target::TargetCpu::X64 | jaic::target::TargetCpu::Arm64
    ) {
        return None;
    }
    let ptr = |ty: TypeId| c.ty.types.pointee(ty).is_some() && c.ty.types.size_of(ty) == 8;
    let i32 = |ty: TypeId| c.ty.types.int_info(ty) == Some((32, true));
    let unix = match c.options.os {
        jaic::target::TargetOs::Linux => matches!(lib.name.as_str(), "c" | "libc"),
        jaic::target::TargetOs::MacOS => matches!(lib.name.as_str(), "c" | "libc" | "System"),
        _ => false,
    };
    let windows = c.options.os == jaic::target::TargetOs::Windows
        && lib.name.eq_ignore_ascii_case("kernel32");
    if foreign.symbol == "fcloseall"
        && unix
        && c.options.os == jaic::target::TargetOs::Linux
        && sig.params.is_empty()
        && i32(sig.returns[0])
    {
        let mut exit = Exit::default();
        exit.global_consumed.insert(NativeKind::Stream);
        return Some(Relation {
            exits: vec![exit],
        });
    }
    let native = match foreign.symbol.as_str() {
        "fopen"
            if unix
                && sig.params.len() == 2
                && sig.params.iter().all(|p| ptr(p.ty))
                && ptr(sig.returns[0]) =>
        {
            Some((NativeKind::Stream, Validity::NonNull))
        }
        "open"
            if unix
                && (sig.params.len() == 2
                    || sig.params.len() == 3
                        && c.ty
                            .types
                            .int_info(sig.params[2].ty)
                            .is_some_and(|(bits, _)| bits == 32))
                && ptr(sig.params[0].ty)
                && i32(sig.params[1].ty)
                // Apple's ARM64 ABI stack-passes an ellipsis mode; a fixed
                // third parameter would instead use a general-purpose register.
                && !(sig.params.len() == 3
                    && !sig.c_varargs
                    && c.options.os == jaic::target::TargetOs::MacOS
                    && c.options.cpu == jaic::target::TargetCpu::Arm64)
                && i32(sig.returns[0]) =>
        {
            Some((NativeKind::Descriptor, Validity::NonNegative))
        }
        "CreateFileW"
            if windows
                && sig.params.len() == 7
                && ptr(sig.params[0].ty)
                && ptr(sig.returns[0])
                && sig.params[1..3]
                    .iter()
                    .all(|p| c.ty.types.int_info(p.ty) == Some((32, false)))
                && ptr(sig.params[3].ty)
                && sig.params[4..6]
                    .iter()
                    .all(|p| c.ty.types.int_info(p.ty) == Some((32, false)))
                && ptr(sig.params[6].ty) =>
        {
            Some((NativeKind::WindowsHandle, Validity::NeitherNullNorMinusOne))
        }
        _ => None,
    };
    if let Some((kind, validity)) = native {
        let mut exit = Exit::default();
        exit.returns.push((
            0,
            None,
            Expr::Fresh(Token {
                kind,
                validity,
                sites: Vec::new(),
            }),
        ));
        return Some(Relation {
            exits: vec![exit],
        });
    }
    let kind = match foreign.symbol.as_str() {
        "fclose"
            if unix && sig.params.len() == 1 && ptr(sig.params[0].ty) && i32(sig.returns[0]) =>
        {
            NativeKind::Stream
        }
        "close"
            if unix && sig.params.len() == 1 && i32(sig.params[0].ty) && i32(sig.returns[0]) =>
        {
            NativeKind::Descriptor
        }
        "CloseHandle"
            if windows && sig.params.len() == 1 && ptr(sig.params[0].ty) && i32(sig.returns[0]) =>
        {
            NativeKind::WindowsHandle
        }
        _ => return None,
    };
    let input = Expr::Input(
        0,
        Slot {
            offset: 0,
            width: c.ty.types.size_of(sig.params[0].ty),
            ty: sig.params[0].ty,
        },
    );
    let mut success = Exit::default();
    success.requirements.push((input.clone(), kind));
    success.consumed.push(HandleEffect {
        value: input.clone(),
        kind: Some(kind),
    });
    if kind == NativeKind::Stream {
        // fclose consumes even when the flush reports an error. Its status is unknown.
        return Some(Relation {
            exits: vec![success],
        });
    }
    let mut failure = Exit::default();
    failure.requirements.push((input.clone(), kind));
    if kind == NativeKind::Descriptor && c.options.os == jaic::target::TargetOs::Linux {
        // Linux releases a valid descriptor before later errors are reported.
        failure.consumed.push(HandleEffect {
            value: input.clone(),
            kind: Some(kind),
        });
    } else {
        // A failure status alone establishes neither closure nor retained liveness.
        // Other close targets and unmodeled Windows handle flags remain unknown.
        failure.uncertain.push(HandleEffect {
            value: input.clone(),
            kind: Some(kind),
        });
    }
    if kind == NativeKind::Descriptor {
        success.returns.push((0, None, Expr::Constant(0)));
        failure.returns.push((0, None, Expr::Constant(u64::MAX)));
    } else {
        // A successful Windows BOOL may be any nonzero value. Keeping its value
        // unknown loses precision safely; it must not become a fabricated integer 1.
        success.returns.push((0, None, Expr::Unknown));
        failure.returns.push((0, None, Expr::Constant(0)));
    }
    Some(Relation {
        exits: vec![success, failure],
    })
}
