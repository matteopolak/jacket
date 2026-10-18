"""Declaration-name audit only; runtime evidence comes from run.py."""
from pathlib import Path

root = Path(__file__).resolve().parents[2]
pairs = [
    ("Ico_File", "ico-file"), ("Icon", "icon"),
    ("Windows_Resources", "windows-resources"), ("Subtitles", "subtitles"),
    ("Simple_Package", "simple-package"),
]
for module, page in pairs:
    spec = (root / f"docs/specs/stdlib/modules/{page}.md").read_text()
    for marker in ("## Public aggregate", "## Public type members", "## Behavioral contract"):
        spec = spec.split(marker)[0]
    expected = {
        line.split(chr(96))[1].split(":", 1)[0].strip()
        for line in spec.splitlines() if line.startswith("- ") and chr(96) in line
    }
    facade = (root / f"stdlib/{module}/module.jai").read_text().split("#scope_file")[0]
    actual = {
        line.split(":", 1)[0].strip()
        for line in facade.splitlines()
        if ":" in line and line.split(":", 1)[0].strip().isidentifier()
    }
    assert expected and expected == actual, (module, expected - actual, actual - expected)
    print(module, len(expected), "reviewed public names; no extra facade names")
