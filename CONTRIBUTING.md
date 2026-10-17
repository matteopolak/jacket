# Contributing

This project does not accept pull requests. Reviewing outside code takes longer than writing the change, so pull requests are closed without review.

Issues are welcome: [open one](https://github.com/matteopolak/jai/issues/new/choose).

- **Bug reports.** A program jaic, jailsp, jaifmt, jailint or the playground gets wrong. The smallest program that shows it is the most useful part; say what Jai does if you know.
- **Feature requests.** Missing language behaviour, stdlib modules, tool features or lints, with the project that needs them.
- **Anything else.** Questions, docs that are wrong or unclear, projects that fail to build. Use a blank issue.

Do not paste or paraphrase code from an official Jai distribution (its modules, `how_to` programs or examples) in issues. Describing what Jai does with your own program is fine.

This is a clean-room implementation. Contributors must not read the official Jai sources at all (compiler, modules, examples or `how_to` programs); behaviour is implemented from [docs/specs](docs/specs/), and unresolved behaviour is recorded in `docs/specs/questions/` rather than guessed. External native libraries' public headers and official documentation may be used for bindings.

Maintainers: the dev commands are [`just` recipes](docs/tools/justfile.md): `just fmt`, `just lint`, `just test`, and `just check` (what CI's format and lint steps run, plus the VS Code generated-file checks). `just hooks` enables a [pre-commit hook](docs/tools/pre-commit-hook.md) that checks the staged Jai and Rust files.
