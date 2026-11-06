//! Actual oracle reducers preserve allocation identity and destination lifetimes.
use super::{Level, TempDir, lint, rendered};

#[test]
fn distinct_fresh_results_survive_replacing_a_pointer_field() {
    let root = TempDir::new("jailint-oracle-fresh-fields");
    let path = root.join("probe.jai");
    let source = r#"#import "Basic";
        Item::struct {value:s64;}
        Holder::struct {item:*Item;}
        make_item::()->*Item{return New(Item);}
        separate::() {
            holder:Holder;holder.item=make_item();first:=holder.item;
            holder.item=make_item();second:=holder.item;
            free(first);free(second);
        }
        same::() {
            holder:Holder;holder.item=make_item();first:=holder.item;second:=first;
            free(first);free(second);
        }
        main::(){separate();same();}
    "#;
    std::fs::write(&path, source).unwrap();
    let (findings, text) = lint(&path, &[("double_free", Level::Warn)]);
    let start = text.find("same::").unwrap();
    let bad = start + text[start..].find("free(second)").unwrap();
    assert_eq!(findings.len(), 1, "{}", rendered(&findings, &text, &path));
    assert!(
        findings[0].start >= bad && findings[0].start < bad + "free(second)".len(),
        "{}",
        rendered(&findings, &text, &path)
    );
}

#[test]
fn owner_cleanup_ends_local_retention_before_its_scope_ends() {
    let root = TempDir::new("jailint-oracle-owner-cleanup");
    let path = root.join("probe.jai");
    let source = r#"#import "Basic";
        Owner::struct {borrowed:*s64;}
        Context::struct {owner:*Owner;}
        G:*Owner;
        close_owner::(owner:*Owner){owner.borrowed=null;free(owner);}
        close_context::(ctx:*Context){old:=ctx.owner;ctx.owner=null;old.borrowed=null;free(old);}
        direct::(){owner:=New(Owner);local:s64;owner.borrowed=*local;owner.borrowed=null;free(owner);}
        wrapped::(){owner:=New(Owner);local:s64;defer close_owner(owner);owner.borrowed=*local;}
        nested::(){ctx:Context;ctx.owner=New(Owner);local:s64;defer close_context(*ctx);ctx.owner.borrowed=*local;}
        retained::(){owner:=New(Owner);local:s64;owner.borrowed=*local;G=owner;}
        main::(){direct();wrapped();nested();retained();}
    "#;
    std::fs::write(&path, source).unwrap();
    let (findings, text) = lint(&path, &[("returned_local_address", Level::Warn)]);
    let start = text.find("retained::").unwrap();
    assert_eq!(findings.len(), 1, "{}", rendered(&findings, &text, &path));
    assert!(
        findings[0].start >= start,
        "{}",
        rendered(&findings, &text, &path)
    );
}

#[test]
fn actual_binding_tree_keeps_the_two_return_allocations_distinct() {
    let path = super::repo().join("stdlib/Bindings_Generator/tests/driven/bindings-tree.jai");
    let (findings, text) = lint(&path, &[("double_free", Level::Warn)]);
    assert!(findings.is_empty(), "{}", rendered(&findings, &text, &path));
}

#[test]
fn actual_binding_sessions_release_retained_inputs_before_local_end() {
    let mut reports = Vec::new();
    for fixture in [
        "bindings-macro-value-printing.jai",
        "generator-model-defaults.jai",
    ] {
        let path = super::repo()
            .join("stdlib/Bindings_Generator/tests/driven")
            .join(fixture);
        let (findings, text) = lint(&path, &[("returned_local_address", Level::Warn)]);
        if !findings.is_empty() {
            reports.push(format!("{fixture}: {}", rendered(&findings, &text, &path)));
        }
    }
    assert!(reports.is_empty(), "{}", reports.join("\n"));
}

#[test]
fn replacing_an_owner_selector_does_not_clean_the_previous_object() {
    let root = TempDir::new("jailint-oracle-owner-replaced");
    let path = root.join("probe.jai");
    let source = r#"#import "Basic";
        Owner::struct {borrowed:*s64;}
        Context::struct {owner:*Owner;}
        retained_old::(){old_owner:Owner;replacement:Owner;ctx:Context;ctx.owner=*old_owner;
            {local:s64;ctx.owner.borrowed=*local;ctx.owner=*replacement;replacement.borrowed=null;}
        }
        main::(){retained_old();}
    "#;
    std::fs::write(&path, source).unwrap();
    let (findings, text) = lint(&path, &[("returned_local_address", Level::Warn)]);
    assert_eq!(findings.len(), 1, "{}", rendered(&findings, &text, &path));
    let borrow = text.find("ctx.owner.borrowed=*local").unwrap();
    assert!(
        findings[0].start >= borrow && findings[0].start < borrow + 25,
        "{}",
        rendered(&findings, &text, &path)
    );
}

#[test]
fn releasing_an_owner_does_not_erase_a_copied_out_borrow() {
    let root = TempDir::new("jailint-oracle-owner-copy-out");
    let path = root.join("probe.jai");
    let source = r#"#import "Basic";
        Owner::struct {borrowed:*s64;}
        KEEP:*s64;
        publish_and_close::(owner:*Owner){KEEP=owner.borrowed;owner.borrowed=null;free(owner);}
        retained::(){owner:=New(Owner);local:s64;owner.borrowed=*local;publish_and_close(owner);}
        main::(){retained();}
    "#;
    std::fs::write(&path, source).unwrap();
    let (findings, text) = lint(&path, &[("returned_local_address", Level::Warn)]);
    assert_eq!(findings.len(), 1, "{}", rendered(&findings, &text, &path));
}

#[test]
fn same_owner_clear_and_replacement_clear_have_distinct_effects() {
    let root = TempDir::new("jailint-oracle-owner-clears");
    let path = root.join("probe.jai");
    let source = r#"#import "Basic";
        Owner::struct {borrowed:*s64;}
        Context::struct {owner:*Owner;}
        G:*Owner;
        KEEP:*s64;
        cleared::(){owner:=New(Owner);local:s64;owner.borrowed=*local;G=owner;owner.borrowed=null;}
        replaced::(){old_owner:Owner;replacement:Owner;ctx:Context;ctx.owner=*old_owner;
            {local:s64;ctx.owner.borrowed=*local;ctx.owner=*replacement;replacement.borrowed=null;}
        }
        copied_out::(){owner:=New(Owner);local:s64;owner.borrowed=*local;
            saved:=owner.borrowed;owner.borrowed=null;KEEP=saved;
        }
        main::(){cleared();replaced();copied_out();}
    "#;
    std::fs::write(&path, source).unwrap();
    let (findings, text) = lint(&path, &[("returned_local_address", Level::Warn)]);
    assert_eq!(findings.len(), 2, "{}", rendered(&findings, &text, &path));
    let replaced = text.find("replaced::").unwrap();
    let copied = text.find("copied_out::").unwrap();
    assert!(
        findings
            .iter()
            .any(|f| f.start >= replaced && f.start < copied),
        "{}",
        rendered(&findings, &text, &path)
    );
    assert!(
        findings.iter().any(|f| f.start >= copied),
        "{}",
        rendered(&findings, &text, &path)
    );
}

#[test]
fn memory_writes_preserve_captures_and_refresh_exposed_pointer_slots() {
    let root = TempDir::new("jailint-oracle-memory-captures");
    let path = root.join("probe.jai");
    let source = r#"#import "Basic";
        Item::struct {value:s64;}
        aliases::(){p:=New(Item);first:=p;slot:=*p;slot.*=New(Item);second:=p;
            free(first);print("%",second.value);print("%",first.value);
            free(second);print("%",second.value);
        }
        partial::(){p:=New(Item);saved:=p;(cast(*u32)*p).*=0;
            print("%",saved.value);free(saved);print("%",saved.value);
        }
        main::(){aliases();partial();}
    "#;
    std::fs::write(&path, source).unwrap();
    let (findings, text) = lint(&path, &[("use_after_free", Level::Warn)]);
    assert_eq!(findings.len(), 3, "{}", rendered(&findings, &text, &path));
    for (marker, expression) in [
        ("free(first)", "first.value"),
        ("free(second)", "second.value"),
        ("free(saved)", "saved.value"),
    ] {
        let start = text.find(marker).unwrap();
        let use_at = start + text[start..].find(expression).unwrap();
        assert!(
            findings
                .iter()
                .any(|f| f.start >= use_at && f.start < use_at + expression.len()),
            "{marker}: {}",
            rendered(&findings, &text, &path)
        );
    }
}


#[test]
fn repeated_native_allocations_do_not_rebind_captured_pointer_epochs() {
    let root = TempDir::new("jailint-oracle-loop-captures");
    let path = root.join("probe.jai");
    let source = r#"#import "Basic";
        libc::#system_library "libc";
        acquire::(bytes:u64)->*void #foreign libc "malloc";
        release::(memory:*void) #foreign libc "free";
        live::(again:bool){current:=acquire(8);if !current return;
            while again {saved:=current;next:=acquire(8);if !next return;current=next;
                release(saved);print("%",(cast(*s64)current).*);
            }
        }
        dead::(again:bool){current:=acquire(8);if !current return;
            while again {saved:=current;next:=acquire(8);if !next return;current=next;
                release(saved);print("%",(cast(*s64)saved).*);
            }
        }
        main::(){}
    "#;
    std::fs::write(&path, source).unwrap();
    let (findings, text) = lint(&path, &[("use_after_free", Level::Warn)]);
    assert_eq!(findings.len(), 1, "{}", rendered(&findings, &text, &path));
    let start = text.find("dead::").unwrap();
    let at = start + text[start..].find("(cast(*s64)saved).*").unwrap();
    assert!(
        findings[0].start >= at && findings[0].start < at + "(cast(*s64)saved).*".len(),
        "{}",
        rendered(&findings, &text, &path)
    );
}

#[test]
fn captured_null_slots_do_not_create_a_released_allocation() {
    let root = TempDir::new("jailint-oracle-null-captures");
    let path = root.join("probe.jai");
    let source = r#"#import "Basic";
        libc::#system_library "libc";
        release::(memory:*void) #foreign libc "free";
        empty::(){first:*void=null;second:*void=null;release(first);release(second);release(first);}
        main::(){empty();}
    "#;
    std::fs::write(&path, source).unwrap();
    let (findings, text) = lint(&path, &[("double_free", Level::Warn)]);
    assert!(findings.is_empty(), "{}", rendered(&findings, &text, &path));
}

#[test]
fn native_null_release_preserves_independent_reset_contract() {
    let root = TempDir::new("jailint-oracle-null-reset-contract");
    let path = root.join("probe.jai");
    let source = r#"#import "Basic";
        libc::#system_library "libc";
        reset_release::(memory:*void) #foreign libc "free"; @ResetsTemporaryStorage
        plain_release::(memory:*void) #foreign libc "free";
        reset::(){temporary:=cast(*u8)talloc(8);reset_release(null);print("%",temporary.*);}
        plain::(){temporary:=cast(*u8)talloc(8);plain_release(null);print("%",temporary.*);}
        main::(){}
    "#;
    std::fs::write(&path, source).unwrap();
    let (findings, text) = lint(&path, &[("use_after_free", Level::Warn)]);
    assert_eq!(findings.len(), 1, "{}", rendered(&findings, &text, &path));
    let start = text.find("reset::").unwrap();
    let at = start + text[start..].find("temporary.*").unwrap();
    assert!(
        findings[0].start >= at && findings[0].start < at + "temporary.*".len(),
        "{}",
        rendered(&findings, &text, &path)
    );
}

#[test]
fn replacing_a_null_pointer_still_reports_a_real_second_release() {
    let root = TempDir::new("jailint-oracle-null-replacement");
    let path = root.join("probe.jai");
    let source = r#"#import "Basic";
        libc::#system_library "libc";
        acquire::(bytes:u64)->*void #foreign libc "malloc";
        release::(memory:*void) #foreign libc "free";
        replaced::(){p:*void=null;p=acquire(8);if !p return;release(p);release(p);}
        main::(){}
    "#;
    std::fs::write(&path, source).unwrap();
    let (findings, text) = lint(&path, &[("double_free", Level::Warn)]);
    assert_eq!(findings.len(), 1, "{}", rendered(&findings, &text, &path));
    let at = text.rfind("release(p)").unwrap();
    assert!(
        findings[0].start >= at && findings[0].start < at + "release(p)".len(),
        "{}",
        rendered(&findings, &text, &path)
    );
}
