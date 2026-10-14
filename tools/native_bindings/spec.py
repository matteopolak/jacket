"""Read reviewed public API pages, without consulting implementation sources.

This is a format parser, not a Jai semantic evaluator. Unknown conditions are
retained for the generator to reject explicitly rather than guessed away.
"""
from dataclasses import dataclass, field, replace
import re


@dataclass(frozen=True)
class Declaration:
    code: str
    condition: str = ""
    line: int = 0

    @property
    def name(self):
        return re.split(r"\s*:(?::|=)?", self.code, maxsplit=1)[0].removeprefix("using ").strip()


@dataclass(frozen=True)
class Shape:
    name: str
    code: str
    condition: str = ""
    line: int = 0
    member_list: bool = False


@dataclass
class Spec:
    declarations: list[Declaration] = field(default_factory=list)
    shapes: list[Shape] = field(default_factory=list)
    parameters: list[str] = field(default_factory=list)
    contextual_blocks: list[str] = field(default_factory=list)
    parameter_blocks: list[str] = field(default_factory=list)


def platform_condition(label):
    label = label.strip()
    option = re.match(r'`(\w+) := (true|false)`;', label)
    if option:
        return ('' if option[2] == 'true' else '!') + option[1]
    if label == "Unix":
        return "OS != .WINDOWS"
    atoms = {"Linux": "OS == .LINUX", "Windows": "OS == .WINDOWS",
             "macOS": "OS == .MACOS", "Android": "OS == .ANDROID",
             "iOS": "OS == .IOS", "ARM64": "CPU == .ARM64", "x64": "CPU == .X64"}
    parts = label.split(", ")
    return " && ".join(atoms.get(p, p) for p in parts)


def shape_blocks(code):
    """A reviewed fence may contain several complete top-level type records."""
    matches = list(re.finditer(r'^(?:using )?([A-Za-z_][A-Za-z_0-9]*)\s*::', code, re.M))
    depth = 0
    start = 0
    blocks = []
    for match in matches:
        prefix = code[start:match.start()]
        depth += prefix.count('{') - prefix.count('}')
        if depth == 0:
            blocks.append((match[1], match.start()))
        start = match.start()
    return [(name, code[begin:blocks[i + 1][1] if i + 1 < len(blocks) else len(code)].rstrip())
            for i, (name, begin) in enumerate(blocks)]


def associate_shared_cpu_variants(spec):
    """Associate deduplicated OS shapes with their declared CPU availability.

    Reviewed inventories list every target declaration, whereas shape sections
    list distinct shapes. A sole CPU-labeled shape for an OS is shared by that
    OS's declared CPUs. Multiple shapes for an OS remain strictly CPU-specific;
    no shape is borrowed across operating systems.
    """
    label = re.compile(r'^OS == (\.\w+) && CPU == \.\w+$')
    for index, shape in enumerate(spec.shapes):
        match = label.fullmatch(shape.condition)
        if not match:
            continue
        siblings = [s for s in spec.shapes if s.name == shape.name and
                    (m := label.fullmatch(s.condition)) and m[1] == match[1]]
        if len(siblings) != 1:
            continue
        declarations = [d for d in spec.declarations if d.name == shape.name and
                        (m := label.fullmatch(d.condition)) and m[1] == match[1]]
        conditions = list(dict.fromkeys(d.condition for d in declarations))
        if len(conditions) > 1:
            spec.shapes[index] = replace(shape, condition=' || '.join('('+c+')' for c in conditions))


def parse_spec(text):
    result = Spec()
    section = ""
    condition = ""
    section_condition = ""
    shape_name = ""
    shape_header = ""
    shape_members = []
    shape_line = 0
    field_condition_open = False
    in_fence = False
    fence = []
    fence_line = 0

    def finish_list(allow_empty=True):
        nonlocal shape_members, shape_header, field_condition_open
        if field_condition_open:
            shape_members.append("}")
            field_condition_open = False
        if shape_header and (shape_members or allow_empty):
            code = shape_header + " {\n" + "\n".join("    " + m.rstrip(";") + ("" if m.endswith("{") or m == "}" else ";") for m in shape_members) + "\n}"
            result.shapes.append(Shape(shape_name, code, condition, shape_line, True))
        shape_members = []
        shape_header = ""

    for number, line in enumerate(text.splitlines(), 1):
        if line.startswith("```"):
            if in_fence:
                code = "\n".join(fence).strip("\n")
                if section in ("Public type members", "Public aggregate shapes and enum values"):
                    for name, block in shape_blocks(code) or [(shape_name, code)]:
                        result.shapes.append(Shape(name, block, condition, fence_line))
                elif "SDK table shape" in section:
                    result.contextual_blocks.append(code)
                elif section == "Module parameters":
                    result.parameter_blocks.append(code)
                in_fence = False
            else:
                finish_list(allow_empty=False)
                in_fence = True
                fence = []
                fence_line = number + 1
            continue
        if in_fence:
            fence.append(line)
            continue
        if line.startswith("## "):
            if line == '## Library and ABI metadata' and shape_header:
                section = 'Library and ABI metadata'
                continue
            finish_list()
            section = line[3:].strip()
            condition = ""
            section_condition = ""
            shape_name = ""
            continue
        if line == '### Target-conditional fields' and shape_header:
            section = 'Public type members'
            continue
        match = re.match(r"### (.+?)-specific (?:type )?declarations", line)
        if match:
            finish_list()
            condition = platform_condition(match[1])
            section_condition = condition
            continue
        match = re.match(r"#{3,4} `([^`]+)`(?: \[([^\]]+)\])?(?: \(([^)]+)\))?", line)
        if match:
            finish_list()
            shape_name = match[1]
            shape_line = number
            condition = match[2] or section_condition
            if match[3]:
                kind, _, config = match[3].partition("; ")
                kind = kind.replace("`", "").removeprefix("empty ").removeprefix("opaque ")
                shape_header = shape_name + " :: " + kind
                if config:
                    condition = config.replace("`", "")
            continue
        match = re.match(r"#### \[(.+)\]", line)
        if match and shape_header:
            cond = match[1]
            if cond.startswith("else of "):
                cond = "!(" + cond.removeprefix("else of ") + ")"
            if field_condition_open:
                shape_members.append("}")
            shape_members.append("#if " + cond + " {")
            field_condition_open = True
            continue
        match = re.match(r"Variant \d+ \(([^)]+)\):", line)
        if match:
            finish_list()
            condition = platform_condition(match[1])
            continue
        match = re.match(r"- Inherited vtable prefix: `([^`]+)`", line)
        if match and shape_header:
            shape_members.append(match[1])
            continue
        match = re.match(r"- (?:\[([^\]]+)\] )?`(.*)`(?: \((.*)\))?$", line)
        if match:
            code = match[2]
            cond = match[1] or condition
            if match[3] and match[3].endswith(' only'):
                cond = platform_condition(match[3].removesuffix(' only'))
            if section == "Public declaration inventory":
                result.declarations.append(Declaration(code, cond, number))
            elif section == "Module parameters":
                result.parameters.append(code)
            elif section == "Public type members" and shape_header:
                shape_members.append(code)
    finish_list()
    associate_shared_cpu_variants(result)
    return result
