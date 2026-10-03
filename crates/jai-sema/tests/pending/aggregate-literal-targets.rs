//! Register after the typed literal parser and canonical indexed plan activate.
use super::check;

#[test]
fn qualified_and_indexed_literals_keep_canonical_defaults() {
    for source in [
        "Holder::struct{values:[3]int=.[10,11,12];tag:int=9;}main::()->int{value:=Holder.{values[1]=20};return value.values[0]+value.values[1]+value.values[2];}",
        "Index::1;Holder::struct{values:[2]int;}answer:Holder:Holder.{values[Index]=42};main::()->int{return answer.values[0]+answer.values[1];}",
        "Payload::union{words:[2]int;number:int;}Message::struct{data:Payload;tag:int=5;}main::()->int{value:=Message.{data.words[1]=37};return value.data.words[0]+value.data.words[1]+value.tag;}",
        "Pair::struct{x:int;y:int=22;}Part::union{pair:Pair;number:int;}Holder::struct{parts:[2]Part;}main::()->int{value:=Holder.{parts[0].pair.x=20,parts[1].number=42};return value.parts[0].pair.x+value.parts[0].pair.y+value.parts[1].number-42;}",
    ] {
        check(source).unwrap_or_else(|error| panic!("{source}\n{error:?}"));
    }
}

#[test]
fn generic_literal_targets_resolve_original_applications() {
    for source in [
        "Packet::struct(T:Type){value:T;values:[2]int=.[13,17];}make::()->Packet(int){return Packet(int).{value=12};}main::()->int{value:=make();return value.value+value.values[0]+value.values[1];}",
        "Packet::struct(T:Type){value:T;values:[2]int=.[13,17];}answer:Packet(int):Packet(int).{value=12};main::()->int{return answer.value+answer.values[0]+answer.values[1];}",
        "Packet::struct(T:Type){value:T;}main::()->int{values:[2]int=.[20,22];view:=([]int).{count=2,data=*values[0]};return view[0]+view[1];}",
    ] {
        check(source).unwrap_or_else(|error| panic!("{source}\n{error:?}"));
    }
}

#[test]
fn literal_paths_reject_conflicts_and_bounds_before_rhs_run() {
    let prefix = "Holder::struct{values:[2]int;}trap::(x:int)->int{return 1/x;}";
    for (body, target, expected) in [
        ("value:=Holder.{values[2]=#run trap(0)};", "2", "count"),
        ("value:=Holder.{values[-1]=#run trap(0)};", "-1", "negative"),
        (
            "index:=0;value:=Holder.{values[index]=#run trap(0)};",
            "index",
            "constant",
        ),
        (
            "value:=Holder.{values=.[20,22],values[0]=#run trap(0)};",
            "values[0]",
            "overlap",
        ),
        (
            "value:=Holder.{values[0]=20,values[0]=#run trap(0)};",
            "values[0]",
            "duplicate",
        ),
    ] {
        let source = format!("{prefix}main::(){{{body}}}");
        let error = check(&source)
            .err()
            .unwrap_or_else(|| panic!("accepted {source}"));
        assert!(error.message.contains(expected), "{source}\n{error:?}");
        let start = source.rfind(target).unwrap();
        assert!(
            error.location.span.start <= start && error.location.span.end >= start + target.len(),
            "{source}\n{error:?}"
        );
    }
}

#[test]
fn indexed_callback_producer_contract_survives_copy() {
    let prefix = "Required::#type(value:int)->int #must;Holder::struct{callbacks:[2]Required;}answer::(value:int)->int{return value;}make::()->Required{return answer;}";
    check(&format!("{prefix}main::()->int{{holder:=Holder.{{callbacks[1]=make()}};copy:=holder;return copy.callbacks[1](value=42);}}")) .unwrap();
    let source = format!(
        "{prefix}main::(){{holder:=Holder.{{callbacks[1]=make()}};copy:=holder;copy.callbacks[1](value=42);}}"
    );
    let error = check(&source)
        .err()
        .expect("required indexed callback cannot be discarded");
    assert!(error.message.contains("#must"), "{error:?}");
    assert_eq!(
        error.location.span.start,
        source.rfind("copy.callbacks[1](value=42)").unwrap()
    );
}
