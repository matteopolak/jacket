# Contributing

This project does not accept pull requests. Reviewing outside code takes longer than writing the change, so pull requests are closed without review.

Issues are welcome: [open one](https://github.com/matteopolak/jai/issues/new/choose).

- **Bug reports.** A program jaic, jailsp, jaifmt, jailint or the playground gets wrong. The smallest program that shows it is the most useful part; say what Jai does if you know.
- **Feature requests.** Missing language behaviour, stdlib modules, tool features or lints, with the project that needs them.
- **Anything else.** Questions, docs that are wrong or unclear, projects that fail to build. Use a blank issue.

Do not paste or paraphrase code from an official Jai distribution (its modules, `how_to` programs or examples) in issues. This is a clean-room implementation. Describing what Jai does with your own program is fine.

Maintainers: `git config core.hooksPath .githooks` enables a [pre-commit hook](docs/tools/pre-commit-hook.md) that checks the staged Jai and Rust files.
