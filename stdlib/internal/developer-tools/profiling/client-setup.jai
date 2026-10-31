// Starts the profiling runtime at the top of `main`.
Prepend_To_Main :: () #expand {
    init_runtime();
}

prepend_main :: #code context._Iprof.Prepend_To_Main();

#load "client-setup-text.jai";
