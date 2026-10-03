// Pure argument ownership/serialization, independent of the native SDK runtime.
// Every returned string and the array itself use the caller's Jai allocator.
build :: (stage: string, args: $T) -> [..] string {
    result: [..] string;
    if args.shader_name.count > 0 add(*result, args.shader_name);

    add(*result, "-HV");
    add(*result, "2021");

    if args.entry_point.count > 0 {
        add(*result, "-E");
        add(*result, args.entry_point);
    }

    add(*result, "-T");
    profile := join(stage, "_", args.shader_model_version);

    Basic.array_add(*result, profile);

    for define: args.defines {
        add(*result, "-D");
        add(*result, define);
    }

    if args.skip_optimization add(*result, "-Od");

    if args.add_debug_info {
        add(*result, "-Zi");
        // A returned stand-alone DXIL object retains its requested debug data.
        if args.spirv_target_env.count == 0 add(*result, "-Qembed_debug");
    }

    if args.warnings_as_error add(*result, "-WX");
    if args.turn_off_validation add(*result, "-Vd");

    if args.spirv_target_env.count > 0 {
        add(*result, "-spirv");
        environment := join("-fspv-target-env=", args.spirv_target_env, "");
        Basic.array_add(*result, environment);
        if args.spirv_generate_non_semanantic_debug_info add(*result, "-fspv-debug=vulkan-with-source");
    }

    for argument: args.extra_args add(*result, argument);

    return result;
}

release :: (arguments: [..] string) {
    for text: arguments Basic.free(text);
    Basic.array_free(arguments);
}

#scope_file
Basic :: #import "Basic";

libc :: #library,system "c";
abort :: () #foreign libc;

add :: (arguments: *[..] string, text: string) {
    Basic.array_add(arguments, Basic.copy_string(text));
}

join :: (a: string, b: string, c: string) -> string {
    builder: Basic.String_Builder;

    Basic.init_string_builder(*builder);
    defer Basic.free_buffers(*builder);
    Basic.append(*builder, a);
    Basic.append(*builder, b);
    Basic.append(*builder, c);
    if builder.failed abort();

    return Basic.builder_to_string(*builder);
}
