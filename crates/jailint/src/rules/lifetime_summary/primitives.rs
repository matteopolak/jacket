//! Ownership at foreign ABI boundaries. Wrapper names play no part in inference.
use super::{Effect, HashMap, Kind, Path, ProcId, Root, Span, Summary};
use jaic::sema::Compiler;

pub(super) fn seed(c: &Compiler, id: ProcId, summary: &mut Summary) {
    let Some(symbol) = super::observable::symbol(c, id) else {
        return;
    };
    let formal = match symbol {
        "free" | "cfree" | "fclose" | "close" | "closesocket" | "munmap" | "VirtualFree"
        | "CloseHandle" | "freeaddrinfo" | "LocalFree" | "GlobalFree" | "RegCloseKey" => 0,
        "VirtualFreeEx" => 1,
        "HeapFree" => 2,
        _ => return,
    };
    if c.pr.procs[id.0 as usize]
        .sig
        .as_ref()
        .is_none_or(|s| s.params.len() <= formal)
    {
        return;
    }
    let effect = Effect {
        target: Path {
            root: Root::Parameter(formal),
            steps: Vec::new(),
        },
        kind: Kind::Free,
        guard: None,
    };
    let witness = vec![c.pr.procs[id.0 as usize].span];
    insert(&mut summary.may, effect.clone(), witness.clone());
    // Fallible native releases may leave a valid resource live on failure.
    // A result-dependent success proof is required before exporting a must release.
    if matches!(symbol, "free" | "cfree" | "fclose" | "freeaddrinfo") {
        insert(&mut summary.guaranteed, effect, witness);
    }
}

fn insert(map: &mut HashMap<Effect, Vec<Span>>, effect: Effect, witness: Vec<Span>) {
    map.insert(effect, witness);
}
