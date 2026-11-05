"""Compare the real Jai formatter consumer with the independent existing Rust schema.

    python3 settings.py --own /path/settings-adapter --rust /path/toml-oracle --report /tmp/settings.json

Compare shared format/doc.format fields, decoded globs and scalar unknown names. Raw override
lexemes are parsed only in this comparison; the actual formatter keeps their original spelling
and source order. Unrelated tools' schemas and nested warning histories are separate contracts.
"""
from __future__ import annotations
import argparse
import hashlib
import json
from pathlib import Path
import subprocess

from semantic import semantic_equal, settings_projection

from toolkit.repo import ROOT


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--own', type=Path, required=True)
    parser.add_argument('--rust', type=Path, required=True)
    parser.add_argument('--report', type=Path, required=True)
    args = parser.parse_args()
    valid = ['', '[format]\n', '[format]\nindent_width=0x2\n', '[format]\nindent_width=1_6\n', 'format.indent_width=0o2\n', 'format={indent_width=0b10}\n', '["format"]\n"indent_width"=+2\n', '[format]\nbrace_style="pres\\u0065rve"\nignore=["vendor\\u002f**", "**/generated_*.jai"]\n', '[doc.format]\nmax_width=0x50\nbrace_style="pres\\u0065rve"\nindent_width=2\n', '[format]\ntyop=1\n', '[doc.format]\ntyop=1\n', (ROOT / 'jacket.toml').read_text()]
    for key in ['blank_lines_between_items', 'blank_line_after_declarations', 'blank_lines_around_blocks', 'blank_line_between_cases', 'blank_line_before_comments', 'blank_line_after_guards', 'blank_line_before_return', 'format_doc_examples']:
        for flag in ('true', 'false'):
            valid.append(f'[format]\n{key}={flag}\n')
    for key, numbers in [('indent_width', [1, 16]), ('case_indent', [0, 16]), ('case_body_indent', [0, 16]), ('max_blank_lines', [0, 100]), ('max_width', [10, 10000])]:
        for number in numbers:
            valid.append(f'[format]\n{key}={number}\n')
    invalid = ['[format]\nindent_width=0\n', '[format]\nindent_width=17\n', '[format]\nindent_width=true\n', '[format]\ncase_indent=-1\n', '[format]\nmax_width=9\n', '[format]\nmax_width=10.0\n', '[format]\nbrace_style="other"\n', '[format]\nblank_line_before_return=1\n', '[format]\nignore=[1]\n', '[format]\nindent_width=2\nindent_width=3\n', '[other]\nbroken=[\n', '[doc.format]\nindent_width=0\n']
    report = dict(own_sha256=hashlib.sha256(args.own.read_bytes()).hexdigest(), rust_sha256=hashlib.sha256(args.rust.read_bytes()).hexdigest(), cases=[])
    failures = 0
    for text, accepted in [(text, True) for text in valid] + [(text, False) for text in invalid]:
        row = dict(input=text, expected_accept=accepted, outcomes={})
        for name, command in [('own', [str(args.own)]), ('rust', [str(args.rust), 'settings'])]:
            result = subprocess.run(command, input=text, capture_output=True, text=True, timeout=10)
            outcome = dict(exit=result.returncode)
            if result.returncode == 0:
                outcome['value'] = settings_projection(json.loads(result.stdout), name == 'rust')
            row['outcomes'][name] = outcome
        own, rust = row['outcomes'].values()
        row['pass'] = semantic_equal(own, rust) and own['exit'] == (0 if accepted else 1)
        failures += not row['pass']
        report['cases'].append(row)
    args.report.parent.mkdir(parents=True, exist_ok=True)
    args.report.write_text(json.dumps(report, indent=2) + '\n')
    print(json.dumps(dict(cases=len(report['cases']), failures=failures, exclusions=0)))
    return int(failures != 0)


if __name__ == '__main__':
    raise SystemExit(main())
