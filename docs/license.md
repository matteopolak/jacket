# License

## What it is

The repository (compiler, language server, wasm build, `stdlib/` and `tools/`) is licensed under the GNU Affero General Public License, version 3 or later (`AGPL-3.0-or-later`, full text in `LICENSE`). Every crate declares it through `license.workspace = true` in its `Cargo.toml`.

## How it works

- **Derivatives stay open.** A modified jaic, or software built from its code, must be distributed under the AGPL, with its source and the existing copyright and license notices.
- **Network use counts.** Running a modified jaic as a service (for example a hosted compiler or playground) obliges you to offer its source to the users of that service. Plain GPLv3 would not.
- **Commercial use is allowed.** No OSI-approved license can forbid it; the AGPL rules out proprietary forks instead.
- **Your programs are yours.** `LICENSE-EXCEPTION` (an additional permission under AGPLv3 section 7) covers `stdlib/`: code they contribute to a compiled program, whether imported, linked, inlined or produced at compile time, does not bring the AGPL along. You may ship the program under any license. The exception does not cover the compiler, the language server, the formatter or other tools, or the stdlib distributed on its own; modified versions of those stay AGPL.

Third-party projects in `corpus/upstream/` are fetched for testing only, are not committed, and keep their own licenses. LLVM (`Apache-2.0 WITH LLVM-exception`) is compatible with the AGPL.

## How to change it

Changing the license or the exception needs the agreement of every copyright holder. If a new directory ships code into compiled programs (a second runtime directory, for example), add it to the list in `LICENSE-EXCEPTION`, `README.md` and here.

## Configuration

`LICENSE` (AGPLv3 text), `LICENSE-EXCEPTION`, and `license` in `[workspace.package]` of the root `Cargo.toml`.

## Dependencies

None.
