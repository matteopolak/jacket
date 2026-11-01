# Friction log

Rough edges hit while writing sitegen. Fixed ones name the commit subject; open ones say so.

## Fixed in the tools

- `make_directory_if_it_does_not_exist("out/a/b/", recursive = true)` failed on Linux and macOS when `out/a` did not exist yet: a name with a trailing slash was taken for its own parent, so no parent was made. Fixed by "fix(file): make parent folders for a recursive name that ends in a slash"; the test is `stdlib/File/tests/make-directory-trailing-slash.jai`. Windows already handled it.

## Open

- `meow_hash` is x86-64 with AES-NI only, so it could not name the assets on an Apple Silicon Mac; `md5` did. `md5` returns the 32-character hex string allocated for each call, which is what is wanted here (the first six digits), but a plain digest of bytes would let a caller choose the encoding. `md5` mirrors the reference module, so its exports are left alone.
- `file_list(path, recursive = true)` returns full paths in directory order, not sorted, with the folder as typed in front. Sitegen sorts them (so builds are reproducible) and cuts the prefix off itself. A sorted listing, or one relative to the folder, would be a new proc in `File_Utilities`.
- `Xar` fixes its capacity in its type: `Xar(Page, 3, 12)` holds at most 8 + 8 + 16 + ... pages and asserts past that. That is plenty for a site, and stable pointers are what the link checker needs, but a growable array with stable elements and no limit does not exist. `Xar` also exports `quick_sort` (a heap sort under that name) next to `Sort.quick_sort`; a file that wants both gets away with it only because it imports `Sort` itself, and the diagnostic for forgetting that import is good (it names both modules).
- `context.allocator = ...` inside a bare `{ }` block was not undone at the end of the block, so a `Flat_Pool` set that way was still the allocator when the pool was destroyed (a crash, then leaks in the caller). `push_allocator` restores at the end of the block. The first version of `build_site` looked right and was not; the docs for `push_allocator` could say that a plain assignment does not scope.
