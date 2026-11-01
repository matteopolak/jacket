# Friction log

Rough edges hit while writing seek. Fixed ones name the commit subject; open ones say so.

## Fixed in the tools

- `Regex.free_regex` freed only the DFA tables, so every `Regex.compile` leaked its parse tree, both programs and its engine, and a pattern that failed to compile leaked what the parser had built. The test framework's leak check reported it on the first test that compiled a pattern. Fixed by "fix(regex): free the parsed pattern and programs in free_regex": the parse and the two programs are allocated from a pool owned by the engine, which `free_regex` releases. The test is `stdlib/Extensions/Regex/tests/free.jai`.

## Open

- A `Thread_Group` has no "wait until idle". The caller counts what `get_completed_work` returns and sleeps a millisecond between polls (the stdlib example does the same). A `wait_for_all(group)` on the group's own semaphores would remove the polling.
- A worker has no start-up hook, and `Thread_Group_Proc` gets the `Thread` rather than the worker's index: seek reaches the index through `thread.worker_info.worker_index` and makes each worker's matcher on its first job. A per-worker `init` callback, or `thread.worker_index`, would read better.
- `Regex.Regex` is not safe to share between threads (its search cache is mutable), so every worker compiles the pattern again. A `Regex.clone(*re)` that shares the program and has its own cache would avoid the compiles; the type's doc already says to compile per thread.
- `visit_files` has a callback and a `user_data`, with no way to stop the walk early or to see why a directory failed (`had_error` only). seek reports "cannot read" without the reason.
- `file_list(path, recursive = true)` has the same limits sitegen found: unsorted, full paths with the folder as typed. seek uses `visit_files` and sorts itself with `Sort.quick_sort`.
- `jacket test` keeps output in one allocation-tracked world, so a test that builds a big string with `join` without `,, temp` fails as a leak. That is the framework doing its job, but the message names `remember` in `tracking.jai`, not the line in the test that allocated; the allocation site would be what a reader needs.
