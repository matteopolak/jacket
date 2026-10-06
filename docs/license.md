# License

## What it is

The repository (compiler, language server, wasm build, `stdlib/` and `tools/`) is licensed under the GNU Affero General Public License, version 3 or later (`AGPL-3.0-or-later`, full text in `LICENSE`). Every crate declares it through `license.workspace = true` in its `Cargo.toml`.

## How it works

- **Derivatives stay open.** A modified jaic, or software built from its code, must be distributed under the AGPL with its source and the existing copyright and license notices (credit).
- **Network use counts.** Running a modified jaic as a service (for example a hosted compiler or playground) obliges you to offer its source to the users of that service. Plain GPLv3 would not.
- **Commercial use is allowed.** No OSI-approved license can forbid it; the AGPL instead makes proprietary forks impossible.
- **Your programs are yours.** Compiling a program does not put it under the AGPL. The question is the stdlib, which compiled programs import and link: until a runtime/stdlib linking exception is decided, a program that ships compiled stdlib code should be treated as a derivative of it. Adding such an exception (like GCC's runtime library exception) is a decision for the copyright holder; see "How to change it".

Third-party projects in `corpus/upstream/` are fetched for testing only, are not committed, and keep their own licenses. LLVM (`Apache-2.0 WITH LLVM-exception`) is compatible with the AGPL.

## How to change it

Changing the license needs the agreement of every copyright holder. To let compiled programs use any license, add a linking exception for `stdlib/` (and `prelude/`) in a `LICENSE-EXCEPTION` file and mention it in `README.md` and here.

## Configuration

`license` in `[workspace.package]` of the root `Cargo.toml`.

## Dependencies

None.
