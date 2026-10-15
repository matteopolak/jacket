#!/usr/bin/env python3
"""Generate available reviewed CodeView constants, recording unresolved aliases."""
import ast
import json
from pathlib import Path
import re
import subprocess

ROOT = Path(__file__).resolve().parents[1]


def main():
    spec = (ROOT / "docs/specs/stdlib/modules/debug-info.md").read_text()
    output = ["// Reviewed public CodeView constants; unresolved dependency aliases are recorded separately.", ""]
    pending = []
    for name in ["Codeview_Leaf_Type", "Codeview_Symbol_Type"]:
        section = spec.split(f"#### `{name}` (enum)", 1)[1].split("#### ", 1)[0]
        expressions = dict(re.findall(r"^- `([A-Za-z_0-9]+)\s*::\s*([^`]+)`", section, re.MULTILINE))
        if len(expressions) < 100:
            raise RuntimeError(f"Incomplete reviewed {name} inventory")
        values = {}

        def resolve(member, active=()):
            if member in values:
                return values[member]
            if member in active:
                raise RuntimeError("Cyclic reviewed enum expression")
            expression = expressions[member]

            def evaluate(node):
                if isinstance(node, ast.Constant) and isinstance(node.value, int):
                    return node.value
                if isinstance(node, ast.Name):
                    return resolve(node.id, (*active, member))
                if isinstance(node, ast.BinOp):
                    left, right = evaluate(node.left), evaluate(node.right)
                    if isinstance(node.op, ast.Add):
                        return left + right
                    if isinstance(node.op, ast.Sub):
                        return left - right
                raise RuntimeError(f"Unsupported reviewed constant expression: {expression}")

            value = evaluate(ast.parse(expression.strip(), mode="eval").body)
            if not 0 <= value <= 0xffff:
                raise RuntimeError(f"Reviewed {name}.{member} exceeds u16 storage")
            values[member] = value
            return value

        output.append(f"{name} :: enum u16 {{")
        for member in sorted(expressions):
            try:
                value = resolve(member)
            except KeyError as missing:
                pending.append({"enum": name, "member": member, "expression": expressions[member], "missing_reviewed_dependency": str(missing.args[0])})
                output.append(f"    // Pending reviewed dependency: {member} :: {expressions[member]}")
                continue
            output.append(f"    {member} :: 0x{value:x};")
        output.extend(["}", ""])
    destination = ROOT / "stdlib/Debug_Info/codeview-values.jai"
    destination.write_text("\n".join(output))
    subprocess.run([ROOT / "target/jaifmt", "--config", ROOT / "jaifmt.toml", destination], check=True)
    (ROOT / "docs/native/developer-tools-codeview-model-pending.json").write_text(json.dumps({"scope": "precise reviewed constant dependencies; no guessed values or additional exports", "pending_aliases": pending}, indent=2) + "\n")


if __name__ == "__main__":
    main()
