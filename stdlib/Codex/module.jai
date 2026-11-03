// Records facts about a build in a `.codex` file: the modules and files it loaded, and per-procedure code metrics such as call counts and loop depth. It can read the file back.
//
// - [get_plugin] is the metaprogram plugin that writes the file when the build finishes (`-export_file <path>`, default `exported.codex`).
// - [make_help] reads a file's bytes into a [Codex_Help] you can walk.
// - [dump_file] prints a summary of a `.codex` file.
//
// ```jai
// ok, help, error := make_help(.[]);
// assert(!ok && error.count > 0);
// ```

// `USAGE_MODE` says what an importer does with the module: record a build (`.WRITE`, the
// default) or read a recorded one back (`.READ`), or both. Every declaration is available in
// every mode.
#module_parameters(USAGE_MODE := Usage_Mode.WRITE) {
    // What an importer uses the module for: `READ` to read a recorded build, `WRITE` to record one.
    Usage_Mode :: enum_flags u32 {
        READ;
        WRITE;
    }
}

Math :: Math_Module;

#load "../internal/developer-tools/codex-inspection.jai";
#load "metrics-model.jai";
#load "../internal/developer-tools/codex-records.jai";
#load "../internal/developer-tools/codex-lifetime.jai";
#load "../internal/developer-tools/codex-wire-values.jai";
#load "../internal/developer-tools/codex-wire-reader.jai";
#load "../internal/developer-tools/codex-plugin.jai";
#load "../internal/developer-tools/codex-metrics.jai";

#scope_file
Math_Module :: #import "Math";
