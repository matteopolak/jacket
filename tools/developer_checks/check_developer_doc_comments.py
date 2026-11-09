#!/usr/bin/env python3
"""Check header comment emission and the same parameter doc model used by hover.

Run with PYTHONPATH=tools python3 tools/developer_checks/check_developer_doc_comments.py.
"""
import json
import subprocess

from developer_checks.dev_common import ENV, JAIC, ROOT, cargo_binary, output_dir


def main():
    source = ROOT / "stdlib/Bindings_Generator/tests/driven/bindings-doc-comments.jai"
    output = output_dir("bindings-doc-comments")
    output.mkdir(parents=True, exist_ok=True)
    for mode in ["interp", "0", "2"]:
        if mode == "interp":
            command = [JAIC, source, "-interp", "--", output]
        else:
            executable = output_dir(f"bindings-doc-comments-o{mode}")
            subprocess.run([JAIC, source, f"-O{mode}", "-exe", executable], env=ENV, cwd=ROOT, check=True)
            command = [executable, output]
        subprocess.run(command, env=ENV, cwd=ROOT, check=True)
    view = output_dir("bindings-doc-comments-view")
    subprocess.run([cargo_binary("jacket", "release"), "doc", output, "--check", "--emit-input", view], env=ENV, cwd=ROOT, check=True)
    modules = [json.loads(p.read_text()) for p in (view / "modules").glob("*.json")]
    item = next(i["content"] for m in modules for i in m["items"] if i["content"]["title"]["label"] == "documented")
    params = item["overloads"][0]["notes"]
    assert {p["name"]: p["doc_html"] for p in params} == {"x": "first value", "y": "second value"}
    narrow = output_dir("bindings-doc-comments-narrow")
    narrow.mkdir(parents=True, exist_ok=True)
    (narrow / "jacket.toml").write_text("[format]\nmax_width = 40\n")
    subprocess.run([JAIC, source, "-interp", "--", narrow, "narrow"], env=ENV, cwd=narrow, check=True)
    text = (narrow / "module.jai").read_text()
    assert "// Slash docs. Bang docs. A paragraph\n// with enough words to wrap at the" in text
    code = False
    for line in text.splitlines():
        if "```" in line:
            code = not code
        elif not code and line.lstrip().startswith("//"):
            assert len(line) <= 40, line
    assert "long_note: s32; // A trailing member comment that spans several words and wraps with the declaration width." in text
    assert "//     int example_with_a_long_name = value + 123456789;" in text
    assert "// int value = 1;\n//     value += 2;" in text
    print("Header doc comments: exact output, interpreter/O0/O2, parameter model and configured width passed")


if __name__ == "__main__":
    main()
