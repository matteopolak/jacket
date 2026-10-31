//! Folding `#if` conditions while skipping inactive blocks.
use super::api::PlatformTarget;
use super::punct::P;
use super::{Tok, Token};

pub(super) fn fold_or(t: &[Token], at: &mut usize, target: PlatformTarget) -> Option<bool> {
    let mut value = fold_and(t, at, target)?;
    while matches!(t.get(*at).map(|x| &x.tok), Some(Tok::Punct(P::OrOr))) {
        *at += 1;
        value |= fold_and(t, at, target)?;
    }
    Some(value)
}

fn fold_and(t: &[Token], at: &mut usize, target: PlatformTarget) -> Option<bool> {
    let mut value = fold_test(t, at, target)?;
    while matches!(t.get(*at).map(|x| &x.tok), Some(Tok::Punct(P::AndAnd))) {
        *at += 1;
        value &= fold_test(t, at, target)?;
    }
    Some(value)
}

/// `!(..)`, `(..)` or `OS == .TAG` / `CPU != .TAG`.
fn fold_test(t: &[Token], at: &mut usize, target: PlatformTarget) -> Option<bool> {
    let tok = |i: usize| t.get(i).map(|x| &x.tok);
    match tok(*at)? {
        Tok::Punct(P::Bang) if matches!(tok(*at + 1), Some(Tok::Punct(P::LParen))) => {
            *at += 1;
            fold_test(t, at, target).map(|v| !v)
        }
        Tok::Punct(P::LParen) => {
            *at += 1;
            let value = fold_or(t, at, target)?;
            if !matches!(tok(*at), Some(Tok::Punct(P::RParen))) {
                return None;
            }
            *at += 1;
            Some(value)
        }
        Tok::Ident(name) => {
            let equal = match tok(*at + 1)? {
                Tok::Punct(P::EqEq) => true,
                Tok::Punct(P::Ne) => false,
                _ => return None,
            };
            if !matches!(tok(*at + 2), Some(Tok::Punct(P::Dot))) {
                return None;
            }
            let Some(Tok::Ident(tag)) = tok(*at + 3) else {
                return None;
            };
            // A further `.member`, call or index would make this something else.
            if matches!(
                tok(*at + 4),
                Some(Tok::Punct(P::Dot | P::LParen | P::LBracket))
            ) {
                return None;
            }
            let value = target.compare(name.as_str(), tag.as_str(), equal)?;
            *at += 4;
            Some(value)
        }
        _ => None,
    }
}
