//! Fresh VM/native witnesses for the coordinated typed literal target wave.
use super::execute;

#[test]
fn indexed_literal_overlays_and_source_order_execute() {
    execute(
        "counter:int;next::(value:int)->int{counter=counter*10+value;return value;}Holder::struct{values:[3]int=.[10,11,12];tag:int;}main::()->int{value:=Holder.{values[1]=next(1),tag=next(2)};return value.values[0]+value.values[1]+value.values[2]+value.tag+counter+5;}",
        &[],
    );
    execute(
        "Pair::struct{x:int;y:int=22;}Part::union{pair:Pair;number:int;}Holder::struct{parts:[2]Part;}main::()->int{value:=Holder.{parts[0].pair.x=20,parts[1].number=42};return value.parts[0].pair.x+value.parts[0].pair.y+value.parts[1].number-42;}",
        &[],
    );
}

#[test]
fn generic_and_structural_literal_targets_execute() {
    execute(
        "Packet::struct(T:Type){value:T;values:[2]int=.[13,17];}answer:Packet(int):Packet(int).{value=12};main::()->int{return answer.value+answer.values[0]+answer.values[1];}",
        &[],
    );
    execute(
        "main::()->int{values:[2]int=.[20,22];view:=([]int).{count=2,data=*values[0]};return view[0]+view[1];}",
        &[],
    );
    execute(
        "main::()->int{values:[2]int=.[20,22];view:=([]int).{2,*values[0]};return view[0]+view[1];}",
        &[],
    );
}

#[test]
fn captured_indexed_callback_runs_once_with_named_required_contract() {
    execute(
        "counter:int;Required::#type(value:int)->int #must;Holder::struct{callbacks:[2]Required;tag:int;}answer::(value:int)->int{return value;}make::()->Required{counter+=1;return answer;}tag::()->int{counter*=10;return 1;}main::()->int{holder:=Holder.{callbacks[1]=make(),tag=tag()};copy:=holder;return copy.callbacks[1](value=31)+counter+copy.tag;}",
        &[],
    );
}

#[test]
fn imported_generic_literal_target_retains_callback_argument_origin() {
    execute(
        "Packets::#import \"Packets\";counter:int;Required::#type(value:int)->int #must;answer::(value:int)->int{return value;}make::()->Required{counter+=1;return answer;}main::()->int{holder:=Packets.Packet(Required).{callback=make()};copy:=holder;return copy.callback(value=41)+counter;}",
        &[(
            "modules/Packets/module.jai",
            "Packet::struct(T:Type){callback:T;}",
        )],
    );
}

#[test]
fn original_index_constant_is_prepared_before_immutable_literal_composition() {
    execute(
        "Index::1;Holder::struct{values:[2]int;}answer:Holder:Holder.{values[Index]=42};main::()->int{return answer.values[0]+answer.values[1];}",
        &[],
    );
}
