# Friction log

## Fixed in the tools

- Meow's public API only returned x64 assembly registers, so framebuffer hashing could not run on this ARM Mac. Fixed by "feat(meow-portable): add a portable byte-oriented Meow hash extension": `hash_bytes` returns the two u64 lanes on any CPU. The scalar implementation is checked against the module's existing independently generated fixture, including tails, blocks, offsets and non-default seeds. Test: `stdlib/Extensions/Meow_Portable/tests/portable.jai`.

## Open

- `Window_Creation` has no portable public close-window procedure. The app ends its process on quit, allowing the OS to reclaim its single window, as the existing game example does. A portable lifetime API belongs in Window_Creation and needs platform tests.
- Native display access is blocked in this sandbox: the real Mac window run reaches `create_window` and fails with `AppKit activation policy failed`. Headless emulation runs; visual output, live controls and audible playback remain unverified here. Smoke reports this known unavailable-display case explicitly.
- An attempted x64 Mac build selected the available ARM audio and crash archives and failed to link. Cross-target native-library provisioning remains a build-tool limitation; the portable Meow API lets this app build and run natively on ARM instead.
- `cargo build -p jaifmt` from the shared brief does not apply to this checkout: it is not a workspace package. The formatter is Jai source in `jaifmt/main.jai`; build it with jaic, then use it directly or through `jacket fmt`.
- `git rebase main` cannot write the original worktree's metadata under the main repository's `.git`. HEAD and main were identical. Commits are made with separate Git metadata under the task scratch directory and delivered in `chip8.bundle`.
