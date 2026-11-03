//! CLI explanations for the lifetime frontends, with minimal examples and proof limits.
pub(super) fn explain(name: &str) -> Option<&'static str> {
    Some(match name {
        "pointer_lifetime" => {
            "pointer_lifetime\n\nTracks the storage generation and logical element a pointer or view borrows. Calls use inferred parameter/field summaries, so wrappers and selected procedure values participate. Header addresses and nested pointed-to allocations have distinct owners.\n\nExample:\n```jai\n#import \"Basic\";\nexample :: () {\n    xs: [..] int;\n    array_add(*xs, 1);\n    xs.count = xs.allocated;\n    p := *xs[0];\n    array_add(*xs, 2);\n    print(\"%\\n\", p.*); // re-derive p after growth\n}\n```\n\nMust mode requires every path and every closed procedure target to invalidate the same borrow. Strict mode accepts a possible invalidating path, including argument-reachable effects of Unknown callees. Configure [lint.lifetimes] mode = \"must\" or \"strict\"; --lifetime-mode wins. Strict witnesses label branch choices and count other abstract bad paths; loops may have infinitely many walks. Unknown element positions stay quiet in must mode.\n"
        }
        "pointer_into_resizable" => {
            "pointer_into_resizable\n\nChecks a pointer derived from resizable-array storage before an intervening growth. It uses the shared storage analysis and keeps this rule's existing default level and local suppression.\n\nExample:\n```jai\np := *items[0];\narray_add(*items, value);\nconsume(p.*); // take the pointer after array_add instead\n```\n\nStore an index or derive the pointer again after growth. A growth-returned pointer belongs to the new generation. An inner allocation survives growth of an outer header array; a pointer to that header does not.\n"
        }
        "remove_in_for" => {
            "remove_in_for\n\nChecks removal or movement that invalidates a forward iteration cursor and can skip elements on a reachable advance. The checker uses selected loop/cleanup edges, so an exit before advancing or an explicit index repair can make the case quiet.\n\nExample:\n```jai\nfor value: items {\n    if unwanted(value) array_remove(*items, it_index);\n}\n```\n\nIterate backward, repair the index, or collect removals for afterward. Pure last-element pop/reset has separate logical-element semantics and is not automatically a skipped-forward-element proof.\n"
        }
        "use_after_free" => {
            "use_after_free\n\nChecks aliases of released allocations and resources, including inferred field releases, foreign validity contracts and deferred cleanup. A started known worker can retain a borrow even when the caller has no later read.\n\nExample:\n```jai\np := New(int);\nfree(p);\nconsume(p.*); // free after the last use\n```\n\nDefault mode needs a definite release. Strict mode can show a conditional release or an Unknown argument-reachable effect. Waiting for successful worker completion ends its captured-data borrow; a false timed wait does not. Caller-owned receiver memory remains distinct from resources it contains.\n"
        }
        "double_free" => {
            "double_free\n\nChecks a second release of the same resource generation without replacement, through copies and inferred wrappers. It preserves the safe removal fix for a definite repeated direct release.\n\nExample:\n```jai\np := New(int);\nfree(p);\nfree(p); // remove this second release\n```\n\nA fresh allocation starts a new generation. Strict may-only diagnostics do not offer the definite-release removal fix.\n"
        }
        "returned_local_address" => {
            "returned_local_address\n\nChecks storage that escapes its stack scope or statement, including by-value parameter/iteration copies, expanded locals, fixed-array views and Any backing storage.\n\nExample:\n```jai\nbad :: () -> *int {\n    local := 42;\n    return *local;\n}\n```\n\nReturn the value, allocate caller-owned storage, or accept an output pointer. A started borrower must finish using stack data before that data's scope ends. Persistent literal and frozen compile-time storage are not local stack storage.\n"
        }
        "temp_storage_escape" => {
            "temp_storage_escape\n\nChecks temporary storage kept in a global/longer-lived destination or returned without a TempAllocates contract. Temporary reset, watermark restore and deferred auto_release_temp participate in the shared release analysis.\n\nExample:\n```jai\nsaved: string;\nremember :: () {\n    saved = tprint(\"%\", 42); // copy into durable storage first\n}\n```\n\nCopy data into owned storage, or declare an intentionally temporary result @TempAllocates so callers preserve its lifetime. Compiler-frozen readable #run results are copied/relocated into persistent globals; opaque addresses alone imply no readable extent.\n"
        }
        _ => return None,
    })
}
