"""Private fixture layout and reviewed target disposition, independent of execution."""

from config import TARGETS
from conditions import evaluate


def private_library_root(work):
    # Relocated module units retain ../../stdlib loads byte-for-byte. Keep the
    # canonical library basename, within a separate private fixture tree.
    return work / "private-units" / "stdlib"


def module_target_condition(module):
    return TARGETS.get(module, TARGETS.get(module.split("/")[0], ""))


def module_result(module, os_name, cpu, result):
    condition = module_target_condition(module)
    supported = not condition or evaluate(
        condition, {"OS": os_name.upper(), "CPU": cpu.upper()}
    )
    if result["exit"] == 0:
        return "passed" if supported else "failed"
    # A required backend failure cannot become an unsupported-target waiver.
    # An unrelated failure on an unavailable target is not a guard proof either.
    guards = (
        module + " requires its documented native platform",
        module + " requires macOS",
    )
    if not supported and any(guard in result["stderr"] for guard in guards):
        return "unsupported"
    return "failed"
