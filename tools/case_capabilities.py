"""Explicit human deferrals; never infer a capability from a compiler failure.

Consumers declare required_capabilities in reviewed case metadata. This table
does not discover/import projects or change compiler/module configuration.
"""

USER_DEFERRED_CAPABILITIES = {
    "native-windows-console-colors": {
        "os": "windows",
        "date": "2026-10-16",
        "reference": "TODO.md#deferred-native-windows-console-colors",
    },
}


def user_deferred_reason(case, target_os):
    reasons = []
    for capability in case.get("required_capabilities", []):
        decision = USER_DEFERRED_CAPABILITIES.get(capability)
        if decision and target_os == decision["os"]:
            reasons.append(
                f"USER_DEFERRED {decision['date']}: {capability} on {target_os} "
                f"({decision['reference']}); platform unavailable, not passed"
            )
    return "; ".join(reasons) or None
