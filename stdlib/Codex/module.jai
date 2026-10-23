// `USAGE_MODE` says what an importer does with the module: record a build (`.WRITE`, the
// default) or read a recorded one back (`.READ`), or both. Every declaration is available in
// every mode.
#module_parameters(USAGE_MODE := Usage_Mode.WRITE) {
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
