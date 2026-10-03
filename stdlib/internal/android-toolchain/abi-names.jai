get_android_project_jni_architecture_name :: (cpu_target: CPU_Tag) -> string {
    if cpu_target == .X64 return "x86_64";
    if cpu_target == .ARM64 return "arm64-v8a";
    B.assert(false, "Android NDK ABI is unavailable for this CPU");
    return "";
}

#scope_file
B :: #import "Basic";
