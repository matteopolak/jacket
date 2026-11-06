//! Caller-visible borrow tests for the actual checked array-add carrier.
use super::{Level, lint, rendered, repo};

#[test]
fn lifetime_array_add_no_growth_preserves_actual_backing_scope() {
    let path = repo().join("tests/lint/pointer_lifetime/resources/stack-array-add-borrows.jai");
    let (lints, text) = lint(
        &path,
        &[
            ("returned_local_address", Level::Warn),
            ("use_after_free", Level::Warn),
        ],
    );
    assert_eq!(lints.len(), 3, "{}", rendered(&lints, &text, &path));
    for procedure in ["return_empty_slot", "return_last_slot"] {
        let start = text.find(procedure).unwrap();
        let body = &text[start..];
        let returned = start + body.find("return array_add").unwrap();
        assert!(
            lints.iter().any(|f| f.rule == "returned_local_address"
                && f.start >= returned
                && f.start < returned + "return array_add(*values);".len()),
            "missing {procedure}: {}",
            rendered(&lints, &text, &path)
        );
    }
    let scope = text.find("use_after_backing_scope").unwrap();
    let use_site = scope + text[scope..].find("stale.*").unwrap();
    assert!(
        lints.iter().any(|f| f.rule == "returned_local_address"
            && f.start >= use_site
            && f.start < use_site + "stale.*".len()),
        "{}",
        rendered(&lints, &text, &path)
    );
    let safe = text.find("return_shared_slot").unwrap();
    assert!(
        lints.iter().all(|f| f.start < safe),
        "{}",
        rendered(&lints, &text, &path)
    );
}

#[test]
fn optional_array_borrow_budget_preserves_unrelated_existing_diagnostics() {
    let root = super::TempDir::new("jailint-array-borrow-budget");
    let path = root.join("probe.jai");
    for copies in [0, 160, 200, 240] {
        let mut source = String::from(
            "#import \"Basic\"; Node::struct {value:s64;} probe::() {back:[4]s64;values:[..]s64;values.data=cast(*s64)*back;values.allocated=4;slot:=array_add(*values);p:=New(Node);free(p);\n",
        );
        for i in 0..copies {
            use std::fmt::Write;
            writeln!(source, "copy_{i}:=p;").unwrap();
        }
        source.push_str("print(\"%\\n\",p.value);}\nmain::(){}\n");
        std::fs::write(&path, &source).unwrap();
        let (findings, text) = lint(&path, &[("use_after_free", Level::Warn)]);
        let use_site = text.find("p.value").unwrap();
        assert!(
            findings.iter().any(|f| f.rule == "use_after_free"
                && f.start >= use_site
                && f.start < use_site + "p.value".len()),
            "copies={copies}: {}",
            rendered(&findings, &text, &path)
        );
    }
}

#[test]
fn certified_append_retains_later_growth_and_release_invalidation() {
    let root = super::TempDir::new("jailint-array-borrow-later-effects");
    let path = root.join("probe.jai");
    let source = r#"#import "Basic";
    later_growth::() {
        backing:[4]s64; values:[..]s64;
        values.data=cast(*s64)*backing; values.allocated=4;
        slot:=array_add(*values);
        array_reserve(*values,100);
        print("%\n",slot.*);
    }
    later_release::() {
        backing:[4]s64; values:[..]s64;
        values.data=cast(*s64)*backing; values.allocated=4;
        slot:=array_add(*values);
        array_free(values);
        print("%\n",slot.*);
    }
    main::(){}
    "#;
    std::fs::write(&path, source).unwrap();
    let (findings, text) = lint(
        &path,
        &[
            ("use_after_free", Level::Warn),
            ("pointer_into_resizable", Level::Warn),
        ],
    );
    for name in ["later_growth", "later_release"] {
        let start = text.find(name).unwrap();
        let use_site = start + text[start..].find("slot.*").unwrap();
        assert!(
            findings
                .iter()
                .any(|f| f.start >= use_site && f.start < use_site + "slot.*".len()),
            "missing {name}: {}",
            rendered(&findings, &text, &path)
        );
    }
}

#[test]
fn repeated_expansion_spans_do_not_suppress_a_later_real_growth() {
    let root = super::TempDir::new("jailint-array-borrow-shared-span");
    let path = root.join("probe.jai");
    let source = r#"#import "Basic";
    append_one::(a:*[..]s64)->*s64 #expand {return array_add(a);}
    probe::() {
        backing:[4]s64; values:[..]s64;
        values.data=cast(*s64)*backing;values.count=2;values.allocated=4;
        first:=array_add(*values);
        second:=append_one(*values);
        third:=append_one(*values);
        print("%\n",first.*);
    }
    main::(){}
    "#;
    std::fs::write(&path, source).unwrap();
    let (findings, text) = lint(&path, &[("pointer_into_resizable", Level::Warn)]);
    let use_site = text.find("first.*").unwrap();
    assert!(
        findings
            .iter()
            .any(|f| f.start >= use_site && f.start < use_site + "first.*".len()),
        "{}",
        rendered(&findings, &text, &path)
    );
}

#[test]
fn append_content_writes_preserve_captured_and_unrelated_pointer_values() {
    let path = repo().join("tests/lint/pointer_lifetime/resources/stack-array-add-contents.jai");

    let (findings, text) = lint(&path, &[("use_after_free", Level::Warn)]);
    assert_eq!(findings.len(), 2, "{}", rendered(&findings, &text, &path));
    for name in ["captured::", "unrelated::"] {
        let start = text.find(name).unwrap();
        let use_site = start + text[start..].find("saved.value").unwrap();
        assert!(
            findings
                .iter()
                .any(|f| f.start >= use_site && f.start < use_site + "saved.value".len()),
            "missing {name}: {}",
            rendered(&findings, &text, &path)
        );
    }
}

#[test]
fn stack_record_append_inherits_the_actual_backing_scope() {
    let path =
        repo().join("tests/lint/pointer_lifetime/resources/stack-record-array-add-borrows.jai");
    let (findings, text) = lint(&path, &[("returned_local_address", Level::Warn)]);
    assert_eq!(findings.len(), 2, "{}", rendered(&findings, &text, &path));
    for needle in ["return array_add", "stale.value"] {
        let at = text.find(needle).unwrap();
        assert!(
            findings
                .iter()
                .any(|f| f.start >= at && f.start < at + needle.len()),
            "missing {needle}: {}",
            rendered(&findings, &text, &path)
        );
    }
}

#[test]
fn native_backed_append_forwards_the_captured_allocation_until_release() {
    let root = super::TempDir::new("jailint-native-array-borrow");
    let path = root.join("probe.jai");
    let source = r#"#import "Basic";
        libc::#system_library "libc";
        acquire::(bytes:u64)->*void #foreign libc "malloc";
        release::(memory:*void) #foreign libc "free";
        other::#system_library "not-c-runtime";
        fake_release::(memory:*void) #foreign other "free";
        closed::() {
            memory:=acquire(64);if !memory return;
            values:[..]s64;values.data=cast(*s64)memory;values.allocated=8;
            slot:=array_add(*values);
            release(memory);
            print("%\n",slot.*);
        }
        live::() {
            memory:=acquire(64);if !memory return;
            values:[..]s64;values.data=cast(*s64)memory;values.allocated=8;
            slot:=array_add(*values);
            print("%\n",slot.*);
        }
        unknown_release::() {
            memory:=acquire(64);if !memory return;
            values:[..]s64;values.data=cast(*s64)memory;values.allocated=8;
            slot:=array_add(*values);
            fake_release(memory);
            print("%\n",slot.*);
        }
        main::(){}
    "#;
    std::fs::write(&path, source).unwrap();
    let (findings, text) = lint(&path, &[("use_after_free", Level::Warn)]);
    let start = text.find("closed::").unwrap();
    let use_site = start + text[start..].find("slot.*").unwrap();
    assert_eq!(findings.len(), 1, "{}", rendered(&findings, &text, &path));
    assert!(
        findings
            .iter()
            .any(|f| f.start >= use_site && f.start < use_site + "slot.*".len()),
        "{}",
        rendered(&findings, &text, &path)
    );
}

#[test]
fn returned_append_does_not_assume_an_incoming_default_provider() {
    let root = super::TempDir::new("jailint-array-provider-open-world");
    let path = root.join("probe.jai");
    let source = r#"#import "Basic";
        State::struct {values:[..]s64;spare:[128]u8;}
        alias_provider::(mode:Allocator_Mode,size:s64,old_size:s64,old_memory:*void,allocator_data:*void)->*void {
            state:=cast(*State)allocator_data;
            if mode==.ALLOCATE || mode==.RESIZE return cast(*void)*state.values.data;
            return null;
        }
        uninvoked::() {
            values:[..]s64;first:=array_add(*values);
            values.count=values.allocated;array_add(*values,5);
            value:=first.*;
        }
        open_world::(values:*[..]s64) {
            first:=array_add(values);
            values.count=values.allocated;array_add(values,5);
            value:=first.*;
        }
        header_alias::() {
            state:State;
            replacement:=context;
            replacement.allocator=.{alias_provider,cast(*void)*state};
            push_context replacement {
                first:=array_add(*state.values);
                state.values.count=state.values.allocated;
                array_add(*state.values,5);
                value:=first.*;
            }
        }
        main::(){header_alias();}
    "#;
    std::fs::write(&path, source).unwrap();
    let (findings, text) = lint(
        &path,
        &[
            ("pointer_into_resizable", Level::Warn),
            ("use_after_free", Level::Warn),
        ],
    );
    assert!(findings.is_empty(), "{}", rendered(&findings, &text, &path));
}

#[test]
fn optional_append_refinement_preserves_trusted_initializer_release_effect() {
    let root = super::TempDir::new("jailint-append-resource-contract");
    let path = root.join("probe.jai");
    let source = r#"#import "Basic";
        libc::#system_library "libc";
        acquire::(bytes:u64)->*void #foreign libc "malloc";
        consume::(p:*s64) {p.*=0;} @FreesArgument
        plain_wrapper::(p:*s64) {consume(p);}
        append::(a:*[..]s64)->*s64 {
            slot:=a.data+a.count;consume(slot);a.count+=1;return slot;
        }
        append_reference::(a:*[..]s64,unused:s64=0)->*s64 {
            slot:=a.data+a.count;consume(slot);a.count+=1;return slot;
        }
        scalar_control::() {
            value:=New(s64);plain_wrapper(value);print("%\n",value.*);
        }
        append_control::() {
            backing:=acquire(32);if !backing return;
            values:[..]s64;values.allocated=4;values.data=cast(*s64)backing;
            saved:=values.data;
            slot:=append(*values);
            print("%\n",saved.*);
        }
        reference_control::() {
            backing:=acquire(32);if !backing return;
            values:[..]s64;values.allocated=4;values.data=cast(*s64)backing;
            saved:=values.data;
            slot:=append_reference(*values);
            print("%\n",saved.*);
        }
        main::(){}
    "#;
    std::fs::write(&path, source).unwrap();
    let (findings, text) = lint(
        &path,
        &[
            ("use_after_free", Level::Warn),
            ("returned_local_address", Level::Warn),
        ],
    );
    for (procedure, needle) in [
        ("scalar_control", "value.*"),
        ("reference_control", "saved.*"),
        ("append_control", "saved.*"),
    ] {
        let start = text.find(procedure).unwrap();
        let use_site = start + text[start..].find(needle).unwrap();
        assert!(
            findings
                .iter()
                .any(|f| f.start >= use_site && f.start < use_site + needle.len()),
            "{procedure}: {}",
            rendered(&findings, &text, &path)
        );
    }
}

#[test]
fn optional_append_refinement_preserves_trusted_global_reset() {
    let root = super::TempDir::new("jailint-append-reset-contract");
    let path = root.join("probe.jai");
    let source = r#"#import "Basic";
        zero::(p:*s64) {p.*=0;}
        append_reset::(a:*[..]s64)->*s64 {
            slot:=a.data+a.count;zero(slot);a.count+=1;return slot;
        } @ResetsTemporaryStorage
        append_reference::(a:*[..]s64,unused:s64=0)->*s64 {
            slot:=a.data+a.count;zero(slot);a.count+=1;return slot;
        } @ResetsTemporaryStorage
        refined::() {
            temporary:=cast(*u8)talloc(8);
            backing:[4]s64;values:[..]s64;
            values.data=cast(*s64)*backing;values.allocated=4;
            slot:=append_reset(*values);
            print("%\n",temporary.*);
        }
        reference::() {
            temporary:=cast(*u8)talloc(8);
            backing:[4]s64;values:[..]s64;
            values.data=cast(*s64)*backing;values.allocated=4;
            slot:=append_reference(*values);
            print("%\n",temporary.*);
        }
        main::(){}
    "#;
    std::fs::write(&path, source).unwrap();
    let (findings, text) = lint(&path, &[("use_after_free", Level::Warn)]);
    for name in ["reference::", "refined::"] {
        let start = text.find(&format!("\n        {name}")).unwrap();
        let use_site = start + text[start..].find("temporary.*").unwrap();
        assert!(
            findings
                .iter()
                .any(|f| f.start >= use_site && f.start < use_site + "temporary.*".len()),
            "{name}: {}",
            rendered(&findings, &text, &path)
        );
    }
}

#[test]
fn actual_aligned_allocation_feeds_returned_append_movement() {
    let root = super::TempDir::new("jailint-actual-aligned-append");
    let path = root.join("probe.jai");
    let source = r#"#import "Basic";
        D::#import "Default_Allocator";
        main::() {
            memory:=D.allocator_proc(.ALLOCATE,64,0,null,null);
            if !memory return;
            values:[..]s64;
            defer array_free(values);
            values.data=cast(*s64)memory;
            values.allocated=8;
            first:=array_add(*values);
            values.count=values.allocated;
            array_add(*values,7);
            print("%\n",first.*);
        }
    "#;
    std::fs::write(&path, source).unwrap();
    let (findings, text) = lint(&path, &[("pointer_into_resizable", Level::Warn)]);
    let use_site = text.find("first.*").unwrap();
    assert_eq!(findings.len(), 1, "{}", rendered(&findings, &text, &path));
    assert!(
        findings
            .iter()
            .any(|f| f.start >= use_site && f.start < use_site + "first.*".len()),
        "{}",
        rendered(&findings, &text, &path)
    );
}
