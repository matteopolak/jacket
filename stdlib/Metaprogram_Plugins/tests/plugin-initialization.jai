// Actual public loader initialization supplies its declared generated body.
// Every original module procedure is still checked with -no_dce.
Matrix_Basic :: #import "Basic";
Matrix_Compiler :: #import "Compiler";
Matrix_Plugins :: #import "Metaprogram_Plugins";

#run,stallable {
    requests: [] Matrix_Plugins.Plugin_To_Create;
    results: [..] *Matrix_Compiler.Metaprogram_Plugin;
    success := Matrix_Plugins.init_plugins(requests, *results, -1);

    Matrix_Basic.assert(success && results.count == 0);
    Matrix_Basic.array_free(results);
}
