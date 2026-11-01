// Standalone entry point retained for `just ui-editor-smoke`.
#load "smoke-session.jai";

main :: () {
    run_smoke(environment_variable("EDITOR_SMOKE_GPU") == "1");
}
