# Friction log

Rough edges hit while writing seek. Fixed ones name the commit subject; open ones say so.

## Fixed in the tools

- A `Thread_Group` had no "wait until idle", so seek polled `get_completed_work` with a one-millisecond sleep, and no per-worker start-up hook. Fixed by "feat(thread): wait for a group to go idle and run a hook as each worker starts": `wait_for_all(*group)` blocks on a condition variable, and `group.worker_start` runs on each worker thread before its first job. The test is `stdlib/Thread/tests/wait-for-all.jai`.

- `Regex.free_regex` freed only the DFA tables, so every `Regex.compile` leaked its parse tree, both programs and its engine, and a pattern that failed to compile leaked what the parser had built. The test framework's leak check reported it on the first test that compiled a pattern. Fixed by "fix(regex): free the parsed pattern and programs in free_regex": the parse and the two programs are allocated from a pool owned by the engine, which `free_regex` releases. The test is `stdlib/Extensions/Regex/tests/free.jai`.

## Open

- `Thread_Group_Proc` gets the `Thread` rather than the worker's index: seek reaches it through `thread.worker_info.worker_index` in each job. (The `worker_start` hook is given the index.)
- `Regex.Regex` is not safe to share between threads (its search cache is mutable), so every worker compiles the pattern again. A `Regex.clone(*re)` that shares the program and has its own cache would avoid the compiles; the type's doc already says to compile per thread.
- `visit_files` has a callback and a `user_data`, with no way to stop the walk early or to see why a directory failed (`had_error` only). seek reports "cannot read" without the reason.
- `file_list(path, recursive = true)` has the same limits sitegen found: unsorted, full paths with the folder as typed. seek uses `visit_files` and sorts itself with `Sort.quick_sort`.
- `jacket test` keeps output in one allocation-tracked world, so a test that builds a big string with `join` without `,, temp` fails as a leak. That is the framework doing its job, but the message names `remember` in `tracking.jai`, not the line in the test that allocated; the allocation site would be what a reader needs.
