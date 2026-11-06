//! Actual ABI context arguments survive executable IR release.
use super::*;
use jaic::ide::lifetimes::LifetimeContextArgument;

#[test]
fn calls_retain_the_exact_evaluated_hidden_context_argument() {
    let mut c = compile(r"
        target::(x:int=argument()+plain())->int{return x;}
        plain::()->int #no_context {return 1;}
        argument::()->int #expand {push_context, defer_pop; return 1;}
        relay::()->int{return target();}
        indirect::(callback:(int)->int)->int #c_call {return callback(1);}
        main::(){relay();}
    ", true, false);
    let mut retained = Vec::new();
    for name in ["relay", "indirect"] {
        let body = bodies(&c).find(|body| c.proc(body.procedure).name.as_str() == name).unwrap();
        let jaic::sema::procs::ProcTarget::Func(id) = c.proc(body.procedure).target.unwrap() else {panic!()};
        let function = c.out.program.funcs[id.0 as usize].as_ref().unwrap();
        assert_eq!(body.call_contexts.len(), if name == "relay" {2} else {1});
        for row in &body.call_contexts {
            let event = &body.blocks[row.block].events[row.event];
            let actual = function.blocks.iter().flat_map(|block| &block.insts).filter_map(|inst| {
                if let jaic::ir::Inst::Call(call) = inst {Some(call)} else {None}
            }).find(|call| match (&call.callee, event) {
                (jaic::ir::Callee::Func(function), Event::Call {callee,..}) => matches!(c.proc(*callee).target, Some(jaic::sema::procs::ProcTarget::Func(actual)) if actual == *function),
                (jaic::ir::Callee::Indirect(value,_), Event::IndirectCall {callee,..}) => value == callee,
                _ => false,
            }).unwrap();
            match row.argument {
                LifetimeContextArgument::Value(value) => {
                    assert_eq!(actual.args.first(), Some(&value));
                    assert!(events(body).any(|event| matches!(event, Event::ContextRead{value:read,..} if *read == value)));
                    if name == "relay" {
                        let flattened = events(body).collect::<Vec<_>>();
                        let read = flattened.iter().position(|event| matches!(event, Event::ContextRead{value:read,..} if *read == value)).unwrap();
                        assert!(flattened[read+1..].iter().any(|event| matches!(event, Event::ContextSet{..})), "default argument macro changes selection after the actual ABI operand was captured");
                    }
                }
                LifetimeContextArgument::Null(value) => {
                    assert_eq!(actual.args.first(), Some(&value));
                    assert!(function.blocks.iter().flat_map(|block| &block.insts).any(|inst| matches!(inst,
                        jaic::ir::Inst::IConst{dst,ty:jaic::ir::Ty::Ptr,value:0} if *dst == value)));
                }
                LifetimeContextArgument::Absent => assert!(actual.args.is_empty()),
            }
        }
        retained.push((body.procedure, body.call_contexts.clone()));
    }
    c.release_lowered_code();
    for (id, rows) in retained {
        assert_eq!(c.ed.facts().unwrap().lifetime_bodies[&id].call_contexts, rows);
    }
}
