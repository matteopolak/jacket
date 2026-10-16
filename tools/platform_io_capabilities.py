"""Reviewed native OS requirements of individual platform-I/O programs.

This is target selection, never an expected compiler-error whitelist. Unknown
dedicated fixtures require an explicit review rather than silently disappearing.
"""

PLATFORM_CONTRACT_CAPABILITIES = {
    "crash-contract": (),
    "file-contract": (),
    "layout-contract": (),
    "process-contract": ("native Process launch and pipes",),
    "runtime-contract": (),
    "startup-arguments-contract": (),
    "system-contract": ("native System account and machine queries",),
    "thread-contract": ("native Thread synchronization and workers",),
    "thread-group-contract": ("native Thread_Group workers",),
}

RUNTIME_CONTRACT_CAPABILITIES = {
    "contracts/process-quoting-contracts": ("native Process module",),
    "contracts/thread-mutex-and-semaphore-contracts": (
        "native Thread synchronization and workers",
    ),
}


def wasm_contract_reason(stem):
    required = PLATFORM_CONTRACT_CAPABILITIES[stem]
    if required:
        return "WASM platform unavailable: requires " + ", ".join(required)
    return None
