# seek

A small grep. It searches files and folders for a pattern on several threads and prints `path:line:col: text` for each line that matches, in path order, so the output is the same on every run.

```sh
jaic examples/seek/main.jai -exe seek
./seek TODO examples                         # a regular expression, every file below examples/
./seek -i -F 'a.b' notes.txt                 # plain text, any case
./seek -l --glob '*.jai' 'Thread_Group' stdlib
./seek -c -j 4 error logs
```

| Flag | Meaning |
| --- | --- |
| `-i` | Ignore case. For `-F` this folds ASCII letters only; patterns use Unicode case folding. |
| `-F` | The pattern is plain text. |
| `-l` | Print only the names of files with a match (wins over `-c`). |
| `-c` | Print `path:N`, the number of matching lines, for every text file (zeros too). |
| `-n`, `--no-line-number` | Print `path: text`, without the line and column. |
| `-g`, `--glob PATTERN` | Only search files that match. Repeat it for several. `*`, `?`, `[a-z]`; a pattern with a `/` is matched against the whole path, `**` crosses folders. |
| `-j`, `--jobs N` | Worker threads (default: one per processor). |

Without paths it searches the current folder. Binary files (a NUL in the first 8 KiB) and folders whose name starts with `.` are skipped; a folder named on the command line is still searched. The column counts characters from 1 and points at the first match on the line.

Exit status, like grep: `0` when a line matched, `1` when none did, `2` for a bad command line, a bad pattern or a path that could not be read (even if something matched).

## Map

- [main.jai](main.jai): The command line (`Extensions/Args`) and the exit status.
- [search/](search): The search as a library. `walk.jai` lists the files, `scan.jai` searches one, `matcher.jai` is the pattern (fixed text or `Extensions/Regex`), `glob.jai` the `--glob` patterns, `run.jai` the thread pool.
- [tests/](tests): `jacket test` in this folder. Each flag, the exit codes, the same output at `-j 1` and `-j 8`, binary files and hidden folders, and the small parts.
- [tools/smoke.sh](tools/smoke.sh): Builds the program and runs it on a small tree. Not part of CI.
- [FRICTION.md](FRICTION.md): What got in the way while writing it.

## How it works

`list_files` walks each path with `visit_files` (hidden folders are told not to descend), filters by glob, sorts the names and drops repeats. Every file becomes a `Job`; the jobs are handed to a `Thread_Group`, and the main thread polls until all have come back.

A worker owns a `Worker`: a `Matcher` (made on its first job, because a compiled regex keeps a search cache that two threads cannot share) and one output buffer. Scanning a file appends its finished output lines to the worker's buffer, and the job remembers where they start and how long they are. When every job is done `join_output` reads the buffers back in job order, which is path order. Which worker ran which file never reaches the output.

The pattern is checked once up front, so a bad one is reported before any file is read.

## Things to try

- Run with `-j 1` and `-j 16` on a big tree and compare the output with `cmp`.
- Add `--only-matching` (the match is already known to `find_in_line`; it needs its end too).
- Search compressed files: `scan_file` reads the whole file, so a decompressing reader (see `Extensions/Deflate`) fits in front of `looks_binary`.
