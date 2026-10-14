# Isolated UI dependency fixtures

Input and Window_Type are record-only test fixtures; Hash_Table provides a small
linear map with the reviewed public storage shape. These exist solely to run
numerics UI mechanisms before the fresh peer foundation supplies these modules.
They do not establish a reference API, native window ABI, or production platform
behavior, and their tests do not count as full final integration. Production UI
imports use the foundation's canonical module names. In particular macOS native
object identity and WASM window policy remain integration gaps.
