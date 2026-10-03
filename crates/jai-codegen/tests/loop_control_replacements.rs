//! Checked source substitutions operate on captured macro storage, never built-in names.
#[path = "support/native_tools.rs"]
mod native_tools;
use std::{
    fs,
    process::Command,
    sync::atomic::{AtomicU64, Ordering},
    time::{Duration, Instant},
};

const FLAGS: &str = "For_Flags::enum_flags u8{POINTER::1; REVERSE::2;}";
static NEXT: AtomicU64 = AtomicU64::new(0);

fn compile(source: &str) -> Result<jai_ir::Program, jai_source::LocatedDiagnostic> {
    compile_with_files(source, &[]).map_err(|(error, _)| error)
}

fn compile_with_files(
    source: &str,
    files: &[(&str, &str)],
) -> Result<jai_ir::Program, (jai_source::LocatedDiagnostic, std::path::PathBuf)> {
    let root = std::env::temp_dir().join(format!(
        "jai-replacement-source-{}-{}",
        std::process::id(),
        NEXT.fetch_add(1, Ordering::Relaxed)
    ));
    fs::create_dir(&root).unwrap();
    let input = root.join("main.jai");
    fs::write(&input, format!("{FLAGS}{source}")).unwrap();
    for (name, text) in files {
        let path = root.join(name);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, text).unwrap();
    }
    let graph = jai_modules::ModuleGraph::load(
        &input,
        jai_modules::GraphOptions {
            import_dirs: vec![root.clone()],
        },
    )
    .unwrap();
    let result = jai_sema::resolve_graph(&graph).map_err(|error| {
        let path = graph
            .sources()
            .get(error.location.source)
            .unwrap()
            .path()
            .to_owned();
        (error, path)
    });
    fs::remove_dir_all(root).unwrap();
    result
}

fn execute(source: &str) {
    let program = compile(source).unwrap();
    execute_program(program);
}

fn execute_program(program: jai_ir::Program) {
    let result = jai_vm::execute(&program, jai_vm::Limits::default());
    assert!(
        matches!(&result.outcome, jai_vm::Outcome::Complete(values) if values[0].integer().unwrap().value()==42),
        "{result:?}"
    );
    let root = std::env::temp_dir().join(format!(
        "jai-replacement-native-{}-{}",
        std::process::id(),
        NEXT.fetch_add(1, Ordering::Relaxed)
    ));
    fs::create_dir(&root).unwrap();
    let context = jai_codegen::Context::create();
    let target = jai_codegen::target::NativeTarget::new().unwrap();
    let module = jai_codegen::lower_for_target(&context, &program, &target).unwrap();
    let object = root.join("program.o");
    target.write_object(&module, &object).unwrap();
    let executable = root.join("program");
    let linked = native_tools::clang_command()
        .arg(&object)
        .arg("-o")
        .arg(&executable)
        .output()
        .unwrap();
    assert!(
        linked.status.success(),
        "{}",
        String::from_utf8_lossy(&linked.stderr)
    );
    let mut process = Command::new(&executable).spawn().unwrap();
    let deadline = Instant::now() + Duration::from_secs(5);
    loop {
        if let Some(status) = process.try_wait().unwrap() {
            assert_eq!(status.code(), Some(42));
            break;
        }
        if Instant::now() >= deadline {
            process.kill().unwrap();
            process.wait().unwrap();
            panic!("generated replacement fixture timed out");
        }
        std::thread::sleep(Duration::from_millis(5));
    }
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn replacement_definition_file_is_distinct_from_inserted_caller_shadowing() {
    let library = format!(
        r#"{FLAGS}
        OFFSET::1;
        Container::struct{{values:[2]int;count:int;}}
        walk::(source:*Container,body:Code,flags:For_Flags)#expand{{
            for value,index:source.values{{
                `it:=value;`it_index:=index;
                #insert(remove={{source.count-=OFFSET;}}) body;
            }}
        }}
    "#
    );
    let source = r#"
        Api::#import "Iter";
        OFFSET::20;
        main::()->int{
            source:Api.Container=.{values=.[20,20],count=4};sum:=0;
            for :Api.walk value:source{sum+=value;remove value;}
            if OFFSET!=20 return 0;
            return sum+source.count;
        }
    "#;
    execute_program(compile_with_files(source, &[("Iter/module.jai", &library)]).unwrap());

    let rejecting = format!(
        r#"{FLAGS}
        Container::struct{{value:int;}}
        walk::(source:Container,body:Code,flags:For_Flags)#expand{{
            for index:0..0{{`it:=source.value;`it_index:=index;
                #insert(remove=#assert(false)) body;
            }}
        }}
    "#
    );
    let caller = "Api::#import\"Iter\";main::(){source:Api.Container;for :Api.walk value:source remove value;}";
    let (error, path) = compile_with_files(caller, &[("Iter/module.jai", &rejecting)]).unwrap_err();
    assert!(path.ends_with("Iter/module.jai"), "{error:?} {path:?}");
    assert_eq!(error.location.span.text(&rejecting), "#assert(false)");
}

#[test]
fn sparse_map_removal_uses_captured_entry_and_publishes_original_container_count() {
    execute(
        r#"
        Entry::struct{hash:int;key:int;value:int;}
        Map::struct{entries:[4]Entry;count:int;}
        for_expansion::(map:*Map,body:Code,flags:For_Flags)#expand{
            for *entry,i:map.entries {
                if entry.hash<2 continue;
                `it_index:=entry.key;
                `it:=entry.value;
                #insert(remove={entry.hash=1;map.count-=1;}) body;
            }
        }
        main::()->int{
            map:Map=.{entries=.[.{hash=2,key=1,value=10},.{hash=0},.{hash=2,key=2,value=14},.{hash=2,key=3,value=16}],count=3};
            sum:=0;
            for value,key:map {sum+=value;if key==2 remove value;}
            if map.entries[2].hash!=1 return 0;
            return sum+map.count;
        }
    "#,
    );
}

#[test]
fn break_replacement_reaches_outer_source_loop_and_crossed_user_cleanup() {
    execute(
        r#"
        Grid::struct{value:int;}
        for_expansion::(grid:Grid,body:Code,flags:For_Flags)#expand{
            for row:0..1 {for column:0..1 {
                `it:=grid.value;
                `it_index:=row*2+column;
                #insert(break=break row) body;
            }}
        }
        main::()->int{
            grid:Grid=.{value=40};sum:=0;
            for value:grid {
                defer sum+=2;
                for nested:0..1 break;
                sum+=value;
                for row:0..1 break value;
            }
            return sum;
        }
    "#,
    );
}

#[test]
fn continue_replacement_uses_definition_storage_and_runs_caller_defers() {
    execute(
        r#"
        Grid::struct{visited:int;}
        for_expansion::(grid:*Grid,body:Code,flags:For_Flags)#expand{
            for row:0..2 {for column:0..1 {
                `it:=row;
                `it_index:=column;
                #insert(continue={grid.visited+=1;continue row;}) body;
            }}
        }
        main::()->int{
            grid:Grid;sum:=0;
            for value:grid {defer sum+=1;sum+=10;continue;}
            return sum+grid.visited*3;
        }
    "#,
    );
}

#[test]
fn unused_assertion_replacements_remain_lazy_and_nested_builtin_removal_keeps_its_owner() {
    execute(
        r#"
        Container::struct{value:int;}
        for_expansion::(source:Container,body:Code,flags:For_Flags)#expand{
            for slot:0..0 {
                `it:=source.value;
                `it_index:=slot;
                #insert(remove=#assert(false),break=#assert(false)) body;
            }
        }
        main::()->int{
            source:Container=.{value=40};backing:[2]int=.[1,2];values:[]int=backing;
            for value:source {
                for value:values {if value==1 remove value;break;}
                source.value=value;
            }
            return source.value+values.count+1;
        }
    "#,
    );
}

#[test]
fn removal_replacement_keeps_macro_index_shadow_and_forward_revisit_behavior() {
    execute(
        r#"
        Container::struct{values:[]int;}
        erase::(source:*Container,index:int){last:=source.values.count-1;source.values[index]=source.values[last];source.values.count-=1;}
        for_expansion::(source:*Container,body:Code,flags:For_Flags)#expand{
            `it_index: int;
            `it: int;
            it_index=-1;
            while it_index<source.values.count-1 {
                it_index+=1;it=source.values[it_index];
                #insert(remove={erase(source,it_index);it_index-=1;}) body;
            }
        }
        main::()->int{
            backing:[4]int=.[2,5,4,7];source:Container=.{values=backing};
            it_index:=20;sum:=0;
            for value,index:source {sum+=value;if(value&1)==0 remove value;}
            return sum+source.values.count+it_index+2;
        }
    "#,
    );
}

#[test]
fn generic_collection_iteration_binds_source_type_and_count_in_definition_scope() {
    execute(
        r#"
        Array::struct(T:Type,N:int){values:[N]T;count:int;}
        for_expansion::(source:*Array($T,$N),body:Code,flags:For_Flags)#expand{
            `it_index:int;
            `it:T;
            it_index=-1;
            while it_index<source.count-1{
                it_index+=1;it=source.values[it_index];
                #insert(remove={source.count-=1;source.values[it_index]=source.values[source.count];it_index-=1;}) body;
            }
            #assert N==4;
        }
        main::()->int{
            T::bool;N::20;
            source:Array(int,4)=.{values=.[2,5,4,7],count=4};sum:=0;
            for value:source{sum+=value;if(value&1)==0 remove value;}
            return sum+source.count+N+2;
        }
    "#,
    );
}

#[test]
fn generic_collection_protocol_rejects_a_distinct_same_shape_nominal_origin() {
    let error = compile(
        r#"
        Array::struct(T:Type){value:T;}
        Other::struct(T:Type){value:T;}
        walk::(source:*Array($T),body:Code,flags:For_Flags)#expand{
            for slot:0..0{`it:=source.value;`it_index:=slot;#insert body;}
        }
        main::(){source:Other(int);for :walk value:source {}}
    "#,
    )
    .unwrap_err();
    assert!(error.message.contains("match"), "{error:?}");
}

#[test]
fn bare_generic_map_formal_retains_concrete_nested_entry_and_value_types() {
    execute(
        r#"
        HashMap::struct(Key:Type,Value:Type){
            Entry::struct{hash:int;key:Key;value:Value;}
            entries:[3]Entry;count:int;
        }
        for_expansion::(map:*HashMap,body:Code,flags:For_Flags)#expand{
            for *entry:map.entries{
                if entry.hash<2 continue;
                `it:=entry.value;
                `it_index:=entry.key;
                #insert(remove={entry.hash=1;map.count-=1;}) body;
            }
        }
        main::()->int{
            map:HashMap(int,int)=.{entries=.[.{hash=2,key=1,value=20},.{hash=0},.{hash=2,key=2,value=20}],count=4};
            sum:=0;
            for value,key:map{sum+=value;remove value;}
            return sum+map.count;
        }
    "#,
    );
}

#[test]
fn distinct_generic_collection_origins_select_their_real_expansion_overloads() {
    execute(
        r#"
        Array::struct(T:Type){values:[2]T;}
        RingBuffer::struct(T:Type){items:[2]T;}
        for_expansion::(source:*Array($T),body:Code,flags:For_Flags)#expand{
            for value,index:source.values{`it:=value;`it_index:=index;#insert body;}
        }
        for_expansion::(source:*RingBuffer($T),body:Code,flags:For_Flags)#expand{
            for value,index:source.items{`it:=value;`it_index:=index;#insert body;}
        }
        main::()->int{
            array:Array(int)=.{values=.[10,10]};ring:RingBuffer(int)=.{items=.[10,10]};sum:=0;
            for value:array sum+=value;
            for value:ring sum+=value;
            return sum+2;
        }
    "#,
    );
}

#[test]
fn exact_source_protocol_wins_over_pointer_adaptation_without_evaluating_source_twice() {
    execute(
        r#"
        Container::struct{value:int;}
        calls:int;
        source::()->Container{calls+=1;return .{value=41};}
        for_expansion::(source:Container,body:Code,flags:For_Flags)#expand{
            `it:=source.value;`it_index:=0;#insert body;
        }
        for_expansion::(source:*Container,body:Code,flags:For_Flags)#expand{
            #assert false;
            `it:=source.value;`it_index:=0;#insert body;
        }
        main::()->int{sum:=0;for value:source() sum+=value;return sum+calls;}
    "#,
    );
}

#[test]
fn protocol_ambiguity_and_definition_errors_do_not_fall_back_to_declaration_order() {
    for (source, message, token) in [
        (
            "Container::struct{} for_expansion::(source:Container,body:Code,flags:For_Flags)#expand{} for_expansion::(source:Container,body:Code,flags:For_Flags)#expand{} main::(){source:Container;for source{}}",
            "ambiguous procedure call",
            "source",
        ),
        (
            "Container::struct{} for_expansion::(source:Container,body:Code,flags:For_Flags)#expand{} for_expansion::(source:Missing,body:Code,flags:For_Flags)#expand{} main::(){source:Container;for source{}}",
            "Missing",
            "source:Missing",
        ),
    ] {
        let error = compile(source).unwrap_err();
        assert!(error.message.contains(message), "{error:?}");
        let original = format!("{FLAGS}{source}");
        assert_eq!(error.location.span.text(&original), token, "{error:?}");
    }
}

#[test]
fn invalid_replacements_report_the_defining_assertion_or_actual_caller_jump() {
    let prefix = "Container::struct{value:int;}for_expansion::(source:Container,body:Code,flags:For_Flags)#expand{for slot:0..0{`it:=source.value;`it_index:=slot;#insert(remove=#assert(false)) body;}}";
    for (source, message, token) in [
        (
            format!("{prefix}main::(){{source:Container;for value:source remove value;}}"),
            "loop-control insertion replacement assertion failed",
            "#assert(false)",
        ),
        (
            format!("{prefix}main::(){{source:Container;for value:source defer remove value;}}"),
            "a deferred body cannot replace an enclosing loop-control statement",
            "remove",
        ),
        (
            format!("{prefix}main::(){{source:Container;for value:source remove unrelated;}}"),
            "remove must name an active enclosing array iterator",
            "remove",
        ),
        (
            "main::(){body::#code 1;value:=#insert(remove={}) body;}".into(),
            "loop-control insertion replacements require statement insertion",
            "#insert(remove={}) body",
        ),
        (
            "main::(){body::#code break;#insert(break={}) body;}".into(),
            "break/continue must target an active enclosing loop",
            "break",
        ),
        (
            "Container::struct{}for_expansion::(#discard source:Container,body:Code,flags:For_Flags)#expand{}main::(){source:Container;for source {}}".into(),
            "discarded custom iteration parameters are not implemented",
            "source",
        ),
    ] {
        let error = compile(&source).unwrap_err();
        assert!(error.message.contains(message), "{error:?}");
        let original = format!("{FLAGS}{source}");
        assert_eq!(error.location.span.text(&original), token, "{error:?}");
    }
}

fn unchanged_focus_ring_buffer() -> String {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .unwrap();
    let path = root.join("corpus/upstream/focus-editor--focus/src/utils/ring_buffer.jai");
    format!("#load \"{}\";", path.to_str().unwrap())
}

#[test]
fn unchanged_full_focus_ring_buffer_executes_generic_wrapped_iteration() {
    let source = format!(
        r#"{}
        main::()->int{{
            ring:Ring_Buffer(int,3);
            ringbuffer_add(*ring,1);ringbuffer_add(*ring,20);
            ringbuffer_add(*ring,21);ringbuffer_add(*ring,22);
            sum:=0;
            for value,index:ring {{if index<1 return 1;sum+=value;}}
            for *value:ring value.*+=1;
            success,item:=ringbuffer_pop(*ring);
            if !success || item!=23 return 2;
            peek:=ringbuffer_peek_pointer(*ring);
            if !peek || peek.*!=22 return 3;
            return sum-ring.count*10-ring.total+2;
        }}
        "#,
        unchanged_focus_ring_buffer()
    );
    execute(&source);
}

#[test]
fn unchanged_full_focus_ring_buffer_rejects_reverse_and_removal_at_definition() {
    for (loop_source, expected) in [
        ("for < value:ring {}", "compile-time assertion failed"),
        (
            "for value:ring remove value;",
            "loop-control insertion replacement assertion failed",
        ),
    ] {
        let source = format!(
            "{}main::(){{ring:Ring_Buffer(int,2);{loop_source}}}",
            unchanged_focus_ring_buffer()
        );
        let (error, path) = compile_with_files(&source, &[]).unwrap_err();
        assert!(error.message.contains(expected), "{error:?}");
        assert!(path.ends_with("src/utils/ring_buffer.jai"), "{path:?}");
        let original = fs::read_to_string(path).unwrap();
        assert!(
            error.location.span.text(&original).starts_with("#assert("),
            "{error:?}"
        );
    }
}

fn supplied_hash_table_iterator_excerpt() -> String {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
    let original = fs::read_to_string(path).unwrap();
    let start = original.find("for_expansion ::").unwrap();
    let end = start + original[start..].find("\r\n}\r\n").unwrap() + 5;
    original[start..end].to_owned()
}

#[test]
fn supplied_constrained_hash_table_iterator_source_removes_and_mutates_entries() {
    // Only the iterator is an unchanged excerpt; this bounded record fixture
    // exercises its real restricted formal without a replacement library.
    let source = format!(
        r#"
        FIRST_VALID_HASH::2;REMOVED_HASH::1;
        Table::struct(Key:Type,Value:Type){{
            Entry::struct{{hash:u32;key:Key;value:Value;}}
            entries:[]Entry;count:int;
        }}
        Concrete_Table::Table(int,int);
        {}
        main::()->int{{
            T::bool;
            entries:[3]Concrete_Table.Entry=.[.{{hash=2,key=1,value=20}},.{{hash=0}},.{{hash=3,key=2,value=21}}];
            table:Concrete_Table=.{{entries=entries,count=2}};sum:=0;
            for value,key:table {{sum+=value;if key==1 remove value;}}
            for *value:table value.*+=1;
            if entries[0].hash!=REMOVED_HASH || entries[2].value!=22 return 1;
            return sum+table.count;
        }}
        "#,
        supplied_hash_table_iterator_excerpt()
    );
    execute(&source);
}

#[test]
fn supplied_hash_table_iterator_rejects_another_same_shape_record_origin() {
    let source = format!(
        "FIRST_VALID_HASH::2;REMOVED_HASH::1;Table::struct(Key:Type,Value:Type){{Entry::struct{{hash:u32;key:Key;value:Value;}}entries:[]Entry;count:int;}}Other::struct(Key:Type,Value:Type){{Entry::struct{{hash:u32;key:Key;value:Value;}}entries:[]Entry;count:int;}}{}main::(){{table:Other(int,int);for table {{}}}}",
        supplied_hash_table_iterator_excerpt()
    );
    let error = compile(&source).unwrap_err();
    assert!(error.message.contains("match"), "{error:?}");
    let original = format!("{FLAGS}{source}");
    assert_eq!(error.location.span.text(&original), "table", "{error:?}");
}

fn unchanged_vk_hash_map_iterator_excerpt() -> String {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../corpus/upstream/ostef--Vk-Engine/Modules/Hash_Map.jai");
    let original = fs::read_to_string(path).unwrap();
    let start = original.find("for_expansion ::").unwrap();
    let end = start + original[start..].find("\n}\n").unwrap() + 3;
    original[start..end].to_owned()
}

#[test]
fn unchanged_vk_hash_map_iterator_uses_pointer_entries_and_original_namespace_types() {
    // The original iterator is preserved byte-for-byte. Storage is an authored
    // bounded fixture, so this does not claim checking the entire Hash_Map root.
    let source = format!(
        r#"
        Hash_Map_First_Occupied_Hash::2;Hash_Map_Removed_Hash::1;
        HashMap::struct(Key:Type,Value:Type){{
            Entry::struct{{hash:u64;key:Key;value:Value;}}
            entries:*Entry;allocated:int;count:int;
        }}
        M::HashMap(int,int);
        {}
        main::()->int{{
            entries:[4]M.Entry=.[.{{hash=2,key=1,value=10}},.{{hash=0}},.{{hash=2,key=2,value=14}},.{{hash=3,key=3,value=16}}];
            map:M=.{{entries=entries.data,allocated=4,count=3}};
            sum:=0;order:=0;
            for < value,key:map {{order=order*10+key;sum+=value;if key==2 remove value;}}
            for *value:map value.*+=1;
            if order!=321 || entries[0].value!=11 || entries[3].value!=17 return 1;
            if entries[2].hash!=Hash_Map_Removed_Hash return 2;
            return sum+map.count;
        }}
        "#,
        unchanged_vk_hash_map_iterator_excerpt()
    );
    execute(&source);
}

fn unchanged_basic_array_removal_excerpt() -> String {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
    let original = fs::read_to_string(path).unwrap();
    let start = original.find("array_unordered_remove_by_value ::").unwrap();
    let end = start + original[start..].find("\r\n}\r\n").unwrap() + 5;
    original[start..end].to_owned()
}

#[test]
fn unchanged_basic_array_removal_executes_generic_default_and_baked_early_exit() {
    let source = format!(
        r#"{}
        main::()->int{{
            backing:[6]int=.[2,4,2,20,2,20];view:[]int=backing;
            removed:=array_unordered_remove_by_value(*view,2);
            one:=array_unordered_remove_by_value(*view,20,stop_after_first=true);
            if removed!=3 || one!=1 || view.count!=2 return 1;
            sum:=0;for value:view sum+=value;
            return sum+removed+one+view.count+12;
        }}
        "#,
        unchanged_basic_array_removal_excerpt()
    );
    execute(&source);
}

#[test]
fn exported_while_binding_keeps_its_physical_target_in_inserted_caller_code() {
    execute(
        r#"
        visit::(body:Code)#expand{
            remaining:=2;
            while `table_while_loop:=remaining {
                remaining-=1;
                #insert body;
            }
        }
        main::()->int{
            sum:=0;
            visit(#code{
                defer sum+=2;
                sum+=20;
                if table_while_loop==1 break table_while_loop;
            });
            return sum-2;
        }
        "#,
    );
}

#[test]
fn exported_while_binding_outside_a_macro_reports_the_marked_name() {
    let source = "main::(){while `target:=1 break target;}";
    let error = compile(source).unwrap_err();
    assert!(
        error
            .message
            .contains("caller exports require an active #expand invocation"),
        "{error:?}"
    );
    let original = format!("{FLAGS}{source}");
    assert_eq!(error.location.span.text(&original), "`target");
}

fn supplied_hash_table_walk_excerpt() -> String {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
    let original = fs::read_to_string(path).unwrap();
    let start = original.find("Walk_Table ::").unwrap();
    let end = start + original[start..].find("\r\n}\r\n").unwrap() + 5;
    original[start..end].to_owned()
}

#[test]
fn supplied_hash_table_walk_source_keeps_caller_bindings_and_exported_loop_target() {
    let library = format!("FIRST_VALID_HASH::2;{}", supplied_hash_table_walk_excerpt());
    let source = r#"
        Hash::#import,file "walk.jai";
        Entry::struct{hash:u32;}
        Map::struct{entries:[]Entry;allocated:int;hash_function:(int)->u32;}
        get_hash::(key:int)->u32{return cast(u32)key;}
        main::()->int{
            entries:[4]Entry=.[.{hash=0},.{hash=0},.{hash=1},.{hash=3}];
            storage:Map=.{entries=entries,allocated=4,hash_function=get_hash};
            table:=*storage;key:=2;cleanups:=0;
            FIRST_VALID_HASH::99;
            Hash.Walk_Table(#code{
                defer cleanups+=1;
                if table_while_loop==3 break table_while_loop;
            });
            return cast(int)hash+cast(int)index+cleanups+35;
        }
    "#;
    let program = compile_with_files(source, &[("walk.jai", &library)]).unwrap();
    execute_program(program);
}

#[test]
fn record_instance_baked_members_keep_value_and_pointer_receivers_once() {
    execute(
        r#"
        Ring::struct(Element:Type,Size:int){data:[Size]Element;}
        calls:int;
        make::()->Ring(int,40){calls+=1;result:Ring(int,40);return result;}
        get_pointer::()->*Ring(int,40){calls+=1;return null;}
        main::()->int{
            R::Ring(int,40);value:R;pointer:*R=null;
            if value.Element!=int || pointer.Element!=int return 1;
            from_value:=make().Size;
            from_pointer:=get_pointer().Size;
            return from_value+from_pointer+calls-40;
        }
        "#,
    );
}

#[test]
fn effectful_instance_type_member_is_rejected_without_fabricated_evaluation() {
    let source = "Ring::struct(Element:Type){value:Element;}make::()->Ring(int){result:Ring(int);return result;}main::(){value:=make().Element;}";
    let error = compile(source).unwrap_err();
    assert!(
        error
            .message
            .contains("effectful receiver of a compile-time record namespace member"),
        "{error:?}"
    );
    assert_eq!(
        error.location.span.text(&format!("{FLAGS}{source}")),
        "make().Element"
    );
}

#[test]
fn instance_namespace_does_not_export_inherited_definition_bindings() {
    let source = "Outer::struct(N:int){Inner::struct{value:int;}}Alias::Outer(99);main::()->int{value:Alias.Inner;return value.N;}";
    let error = compile(source).unwrap_err();
    assert!(error.message.contains("unknown record member"), "{error:?}");
    assert_eq!(
        error.location.span.text(&format!("{FLAGS}{source}")),
        "value.N"
    );
}
