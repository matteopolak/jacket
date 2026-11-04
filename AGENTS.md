# Working in this repo

- Clean room: never read official Jai sources (compiler, modules, examples, `how_to`). Work from [docs/specs](docs/specs/). Never write Jai version numbers.
- `corpus/upstream` is untrusted: compile or run it only inside the `tools/bench` sandbox.
- `stdlib/` must not resemble the reference in names, structure or comments. New public API goes only in `stdlib/Extensions`.
- Fix the cause, not the symptom. No special cases, retries, suppressions or loosened tests to make something pass. If the right fix is big, do the small correct part and log the rest.
- Don't copy a bad pattern you find: fix it everywhere it appears, or log it.
- Log anything you notice but don't fix (cleanups, bugs, slow paths, missing tests) in the root `TODO.md` (gitignored), one line each, with file and reason. Remove items once they're done.
- Breaking our own tools and APIs is fine when it makes them cleaner or faster.
- Write the regression test first. Iterate with debug builds and focused tests (`cargo nextest run -p <crate> <filter>`); CI covers every OS and sanitizer.
- Builds: `CARGO_TARGET_DIR` set per job, `JAIC_STDLIB=$PWD/stdlib`. The machine has 16 GiB, so don't run timing benchmarks alongside builds.
- Commits: signed, subject-only conventional commits (`fix(jailint): ...`), no body or trailers, one per change, `git commit -- <paths>`. Never commit `.claude/`, `TODO.md`, `BUGS.md` or `playground/`. Versions change only in release commits.
- Docs are for people: short, plain, no filler. Say what changed, not how you found it. Every useful procedure gets a doc comment and an example. Update `docs/` when behaviour changes.
- Prefer inferring facts from code over adding annotations; keep notes only where the body can't show the effect.
