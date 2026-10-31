# License

The repository (compiler, language server, wasm build, `stdlib/` and `tools/`) is licensed under the GNU Affero General Public License, version 3 or later (`AGPL-3.0-or-later`, full text in `LICENSE`). Every crate declares it with `license.workspace = true`.

- **Derivatives stay open.** A modified jaic, or software built from its code, must be distributed under the AGPL with its source and notices.
- **Network use counts.** Running a modified jaic as a service (a hosted compiler or playground) obliges you to offer its source to that service's users.
- **Commercial use is allowed.** The AGPL rules out proprietary forks instead.
- **Your programs are yours.** `LICENSE-EXCEPTION` (an additional permission under AGPLv3 section 7) covers `stdlib/` including `stdlib/prelude/`: code it contributes to a compiled program, whether imported, linked, inlined or produced at compile time, does not bring the AGPL along. Ship the program under any license. The exception does not cover the compiler, language server, formatter or other tools, or the stdlib distributed alone.

Third-party projects in `corpus/upstream/` are fetched for testing only, are not committed, and keep their own licenses. LLVM (`Apache-2.0 WITH LLVM-exception`) is compatible.

## How to change it

Changing the license or the exception needs every copyright holder's agreement. If a new directory ships code into compiled programs, add it to `LICENSE-EXCEPTION`, `README.md` and here. The license lives in `LICENSE`, `LICENSE-EXCEPTION` and `[workspace.package]` of the root `Cargo.toml`.
