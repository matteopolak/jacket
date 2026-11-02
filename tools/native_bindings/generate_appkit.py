"""Generate AppKit message forwarding from the reviewed public contract.

Only docs/specs is an API input. The native selector spelling addSubview:
comes from Apple's public NSView header. Reviewed native-void/public-id
operations dispatch as void and return the explicitly unspecified null slot.
"""

import re
from pathlib import Path
from generators.jai_codegen import emit_generated, generator_parser, write_or_check
from native.provided_libs import EXTERNAL
from toolkit.repo import ROOT

SPEC = ROOT / "docs/specs/stdlib/modules/appkit.md"
OUT = ROOT / "stdlib/Objective_C/AppKit/generated.jai"
OUTPUTS = [
    "stdlib/Objective_C/AppKit/generated.jai",
    "stdlib/Objective_C/AppKit/module.jai",
]
FORMATTED = True
VOID_RESULT = {
    ("NSApplication", n) for n in ("setDelegate", "finishLaunching", "setMainMenu")
}
VOID_RESULT.add(("NSOpenGLContext", "update"))
FOUNDATION = (
    "NSObject",
    "id",
    "BOOL",
    "NSRect",
    "NSPoint",
    "CGFloat",
    "NSInteger",
    "NSUInteger",
    "NSDate",
    "NSRunLoopMode",
    "NSDictionary",
    "CALayer",
    "NSArray",
    "NSPasteboardType",
    "NSString",
    "NSTimeInterval",
    "Selector",
)


def qualify(text):
    for name in FOUNDATION:
        text = re.sub(r"\b" + name + r"\b", "O." + name, text)
    return text


def enum(name, kind, base, members):
    return (
        f"using {name} :: {kind} {base} {{\n"
        + "".join(f"    {key} :: {value};\n" for key, value in members)
        + "}\n\n"
    )


def method(cls, name, params, result="", no_context=False):
    signature = f"    {name} :: ({qualify(params)})"
    if result:
        signature += " -> " + qualify(result)
    if no_context:
        signature += " #no_context"
    parts = re.split(r",\s*(?![^()]*\))", params) if params else []
    arguments = [p.split(":", 1) for p in parts]
    instance = bool(arguments and arguments[0][0] == "self")
    receiver = "self" if instance else f'binding_class("{cls}")'
    arguments = arguments[1:] if instance else arguments
    selector = name
    if arguments:
        selector += ":" + "".join(p[0] + ":" for p in arguments[1:])
    selector = {
        "addSubView:": "addSubview:",
        "mouse_inRect:inRect:": "mouse:inRect:",
    }.get(selector, selector)
    # Reviewed legacy self-taking class properties retain the public parameter,
    # but never use it as an Objective-C receiver (including when self is null).
    if cls == "NSEvent" and name in {
        "keyRepeatDelay",
        "keyRepeatInterval",
        "pressedMouseButtons",
    }:
        receiver = 'binding_class("NSEvent")'
    native_params = ["receiver: *void", "selector: O.Selector"]
    native_params += [qualify(key + ":" + value) for key, value in arguments]
    void_result = (cls, name) in VOID_RESULT
    native_result = " -> " + qualify(result) if result and not void_result else ""
    body = signature + " {\n"
    body += (
        "        Send :: #type ("
        + ", ".join(native_params)
        + ")"
        + native_result
        + " #c_call;\n"
    )
    if result and not void_result:
        body += "        send: Send;\n"
        body += f"        #if CPU == .X64 && size_of({qualify(result)}) > 16 {{\n"
        body += "            send = cast(Send) binding_send_stret;\n        } else {\n"
        body += "            send = cast(Send) binding_send;\n        }\n"
    else:
        body += "        send := cast(Send) binding_send;\n"
    actual = [receiver, 'binding_selector("' + selector + '")'] + [
        p[0] for p in arguments
    ]
    call = "send(" + ", ".join(actual) + ")"
    if cls == "NSApplication" and name == "sharedApplication":
        body += "        NSApp = " + call + ";\n        return NSApp;\n"
    else:
        body += (
            "        "
            + ("return " if result and not void_result else "")
            + call
            + ";\n"
        )
        if void_result:
            body += "        // Reviewed unspecified Jai id slot; native operation returns void.\n        return null;\n"
    return body + "    }\n\n"


def render_manual_aliases(spec, source):
    """Insert reviewed aliases into existing public aggregates, without wrappers."""
    declarations = re.findall(
        r"The `(\w+)` aggregate also publicly aliases `(\w+)\.(\w+)`", spec
    )
    for owner, target, name in dict.fromkeys(declarations):
        aggregate = re.search(r"(?m)^" + re.escape(owner) + r" :: struct\s*\{", source)
        if aggregate is None:
            raise ValueError("Missing public alias owner " + owner)
        opening = aggregate.end() - 1
        depth = 0
        closing = None
        # Procedure bodies and braces in comments/strings cannot terminate the
        # owning record. No new record, field or forwarding procedure is made.
        tokens = re.finditer(
            r'//[^\n]*|/\*.*?\*/|"(?:\\.|[^"\\])*"|[{}]', source[opening:], re.S
        )
        for token in tokens:
            if token[0] == "{":
                depth += 1
            elif token[0] == "}":
                depth -= 1
                if depth == 0:
                    closing = opening + token.start()
                    break
        if closing is None:
            raise ValueError("Unterminated public alias owner " + owner)
        body = source[opening + 1 : closing]
        member = r"(?m)^\s*" + re.escape(name) + r"\s*::"
        if re.search(member, body):
            if not re.search(
                member + r"\s*" + re.escape(target + "." + name) + r"\s*;", body
            ):
                raise ValueError("Conflicting public alias " + owner + "." + name)
            continue
        line = "    " + name + " :: " + target + "." + name + ";\n"
        if not source[:closing].endswith("\n"):
            line = "\n" + line
        source = source[:closing] + line + source[closing:]
    return source


def render_generated():
    spec = SPEC.read_text()
    out = "// AppKit declarations; the hand-written API is in module.jai.\n\n"
    for name, value in re.findall(
        r"`(NSOpenGL\w+): NSOpenGLPixelFormatAttribute: ([\dxabcdef]+)`", spec
    ):
        out += f"{name}: NSOpenGLPixelFormatAttribute: {value};\n"
    out += "\nNSOpenGLPixelFormatAttribute :: u32;\n\n"
    for name, base in (
        ("NSOpenGLContextParameter", "O.NSInteger"),
        ("NSApplicationActivationPolicy", "O.NSInteger"),
        ("NSWindowStyleMask", "O.NSUInteger"),
        ("NSEventType", "O.NSUInteger"),
    ):
        members = re.findall(r"`(" + name + r"\w+)=(\d+(?:<<\d+)?)`?", spec)
        members = list(dict.fromkeys(members))
        out += enum(
            name, "enum_flags" if name == "NSWindowStyleMask" else "enum", base, members
        )
    out += enum(
        "NSBackingStoreType",
        "enum",
        "O.NSUInteger",
        [
            ("NSBackingStore" + n, i)
            for i, n in enumerate(("Retained", "Nonretained", "Buffered"))
        ],
    )
    event_values = list(dict.fromkeys(re.findall(r"`NSEventType(\w+)=(\d+)`", spec)))
    out += enum(
        "NSEventMask",
        "enum",
        "u64",
        [("NSEventMask" + n, "1 << " + v) for n, v in event_values]
        + [("NSEventMaskAny", "0xffffffffffffffff")],
    )
    modifiers = re.findall(r"`(NSEventModifierFlag\w+)=(\d+<<\d+|0x[0-9a-f]+)`", spec)
    out += enum("NSEventModifierFlags", "enum_flags", "O.NSUInteger", modifiers)
    presentation = spec.split("with members\n`NSApplicationPresentationDefault=0`", 1)[
        1
    ].split("; nonzero members", 1)[0]
    names = re.findall(r"`(NSApplicationPresentation\w+)`", presentation)
    out += enum(
        "NSApplicationPresentationOptions",
        "enum",
        "O.NSUInteger",
        [("NSApplicationPresentationDefault", "0")]
        + [(n, f"1 << {i}") for i, n in enumerate(names)],
    )
    out += enum(
        "NSRequestUserAttentionType",
        "enum",
        "O.NSUInteger",
        [("NSCriticalRequest", 0), ("NSInformationalRequest", 10)],
    )

    def declared_block(marker):
        section = spec.split(marker, 1)[1]
        return section.split("```jai\n", 1)[1].split("```", 1)[0]

    classes = ("NSApplication", "NSView", "NSOpenGLContext", "NSOpenGLPixelFormat")
    message_blocks = [declared_block("`" + cls + "` embeds") for cls in classes]
    for cls, block in zip(
        classes,
        message_blocks,
        strict=True,
    ):
        out += cls + " :: struct {\n    using #as super: O.NSObject;\n"
        if cls == "NSView":
            out += "    pad: [152 - size_of(O.NSObject)] u8;\n"
        out += "\n"
        for line in block.strip().splitlines():
            match = re.fullmatch(
                r"(\w+) :: \((.*)\)(?: -> (.*?))?( #no_context)?", line
            )
            if not match:
                raise ValueError(line)
            name, params, result, no_context = match.groups()
            out += method(cls, name, params, result or "", bool(no_context))
        out += "}\n\n"
    for cls, body in re.findall(
        r"(NSMenu(?:Item)?) :: struct \{(.*?)\}",
        declared_block("`NSMenu` and `NSMenuItem`"),
        re.S,
    ):
        out += cls + " :: struct {\n    using #as super: O.NSObject;\n"
        for name, params, result in re.findall(
            r"(\w+) :: \((.*?)\)(?: -> ([^\n]+))?", body
        ):
            out += method(cls, name, params, result)
        out += "}\n\n"
    out += "NSScreen :: struct {\n    using #as super: O.NSObject;\n"
    out += method("NSScreen", "mainScreen", "", "*NSScreen")
    out += "    frame :: NSView.frame;\n    backingScaleFactor :: NSView.backingScaleFactor;\n}\n\n"
    out += "NSApplicationDelegate :: struct {\n    using #as nsobject: O.NSObject;\n} @ObjCProtocol\n\n"
    out += "NSEvent :: struct {\n    using #as super: O.NSObject;\n"
    out += method("NSEvent", "mouseLocation", "", "NSPoint")
    event_methods = spec.split("`NSEvent` embeds `NSObject` and exposes", 1)[1].split(
        "The modifier flag", 1
    )[0]
    for name, result in re.findall(
        r"`(\w+)\(self: \*NSEvent\) -> (\*?\w+)`", event_methods
    ):
        out += method("NSEvent", name, "self: *NSEvent", result)
    for name in ("deltaX", "deltaY", "deltaZ"):
        out += method("NSEvent", name, "self: *NSEvent", "CGFloat")
    out += "}\n\n"
    cg_section = spec.split("`CGWindowLevel :: s32`", 1)[1]
    keys = re.findall(
        r"`(kCG\w+WindowLevelKey|kCGBackstopMenuLevelKey|kCGNumberOfWindowLevelKeys)(?: :: 0)?`",
        cg_section.split("`CGWindowLevelForKey", 1)[0],
    )
    out += "CGWindowLevel :: s32;\n" + enum(
        "CGWindowLevelKey",
        "enum",
        "s32",
        list(zip(keys, range(len(keys)), strict=True)),
    )
    globals_section = cg_section.split("The mutable `CGWindowLevel` globals are", 1)[
        1
    ].split("Their values", 1)[0]
    globals_ = re.findall(r"`(kCG\w+Level)`", globals_section)
    for name in globals_:
        out += name + ": CGWindowLevel;\n"
    out += "\nCGWindowLevelForKey :: (key: CGWindowLevelKey) -> CGWindowLevel #foreign CoreGraphics;\n\n"
    out += "init_core_graphics_runtime_constants :: () {\n    if kCGBaseWindowLevel return;\n"
    for name in globals_:
        out += f"    {name} = CGWindowLevelForKey(.{name}Key);\n"
    out += "}\n\n"
    out += "init_app_kit :: () {\n    if !selectors.characters initialize_selectors();\n    init_core_graphics_runtime_constants();\n}\n\n"
    selector_names = list(
        dict.fromkeys(re.findall(r'binding_selector\("([^"]+)"\)', out))
    )
    manual = (OUT.parent / "module.jai").read_text()
    selector_names += [
        name
        for name in re.findall(r'O.selector\("([^"]+)"\)', manual)
        if name not in selector_names
    ]
    out += "#scope_file\nSelectors :: struct {\n"
    for name in selector_names:
        out += "    " + name.replace(":", "_") + ": O.Selector;\n"
    out += "}\nselectors: Selectors;\ninitialize_selectors :: () {\n"
    for name in selector_names:
        out += f'    selectors.{name.replace(":", "_")} = binding_selector("{name}");\n'
    out += "}\n"
    out += """
#scope_file
O :: #import "Objective_C";
libobjc :: #library,system "objc"; """ + EXTERNAL + """
CoreGraphics :: #library,system "CoreGraphics"; """ + EXTERNAL + """
binding_class :: (name: *u8) -> *void #foreign libobjc "objc_getClass";
binding_selector :: (name: *u8) -> O.Selector #foreign libobjc "sel_registerName";
binding_send :: () #foreign libobjc "objc_msgSend";
#if CPU == .X64 binding_send_stret :: () #foreign libobjc "objc_msgSend_stret";
"""
    return out


def main():
    parser = generator_parser(__doc__.split("\n")[0])
    parser.add_argument("--output", type=Path, default=OUT)
    parser.add_argument(
        "--module-output",
        type=Path,
        help="Emit reviewed aliases in the authored facade; external output leaves inputs unchanged",
    )
    args = parser.parse_args()
    emit_generated(args.output, render_generated(), args.formatter, args.check)
    module_output = args.module_output
    if module_output is None and args.output == OUT:
        module_output = OUT.parent / "module.jai"
    if module_output is not None:
        write_or_check(
            module_output,
            render_manual_aliases(
                SPEC.read_text(), (OUT.parent / "module.jai").read_text()
            ),
            args.check,
        )


if __name__ == "__main__":
    main()
