# Friction log

Rough edges hit while writing seek. Fixed ones name the commit subject; open ones say so.

## Fixed in the tools

- A `Thread_Group` had no "wait until idle", so seek polled `get_completed_work` with a one-millisecond sleep, and no per-worker start-up hook. Fixed by "feat(thread): wait for a group to go idle and run a hook as each worker starts": `wait_for_all(*group)` blocks on a condition variable, and `group.worker_start` runs on each worker thread before its first job. The test is `stdlib/Thread/tests/wait-for-all.jai`.

- `Regex.free_regex` freed only the DFA tables, so every `Regex.compile` leaked its parse tree, both programs and its engine, and a pattern that failed to compile leaked what the parser had built. The test framework's leak check reported it on the first test that compiled a pattern. Fixed by "fix(regex): free the parsed pattern and programs in free_regex": the parse and the two programs are allocated from a pool owned by the engine, which `free_regex` releases. The test is `stdlib/Extensions/Regex/tests/free.jai`.

- Every seek worker compiled the pattern again, since a `Regex.Regex` has a mutable search cache. Fixed by "feat(regex): clone a compiled pattern for another thread": `Regex.clone(*re)` shares the programs and has a cache of its own, and seek compiles once and clones per worker. The test is `stdlib/Extensions/Regex/tests/clone.jai`.

- A leak found by `jacket test` was reported at `remember` in `tracking.jai` when the allocating call went through more than a few library procedures (`join`). Fixed by "fix(testing): report a leak at the line in the test that allocated": the allocation records its first frame outside the standard library when it is made. The test is `crates/jacket/tests/leak_report.rs`.

## Open

- `Thread_Group_Proc` gets the `Thread` rather than the worker's index: seek reaches it through `thread.worker_info.worker_index` in each job. (The `worker_start` hook is given the index.)
- `visit_files` has a callback and a `user_data`, with no way to stop the walk early or to see why a directory failed (`had_error` only). seek reports "cannot read" without the reason.
- `file_list(path, recursive = true)` has the same limits sitegen found: unsorted, full paths with the folder as typed. seek uses `visit_files` and sorts itself with `Sort.quick_sort`.
