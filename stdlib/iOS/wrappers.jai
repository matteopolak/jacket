#if OS == .IOS {
    get_ios_target_triple :: (cpu: CPU_Tag) -> string {
        return Toolchain.target_triple(cpu);
    }

    get_ios_sdk_path :: () -> string {
        return Toolchain.ios_sdk_path();
    }

    get_ios_simulator_sdk_path :: () -> string {
        return Toolchain.simulator_sdk_path();
    }
}

#scope_file
#if OS == .IOS Toolchain :: #import "internal/bindings/apple-toolchain";
