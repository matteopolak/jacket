"""Generate AppKit message forwarding from the reviewed public contract.

Only docs/specs is an API input. The native selector spelling addSubview:
comes from Apple's public NSView header. Reviewed native-void/public-id
operations dispatch as void and return the explicitly unspecified null slot.
"""

import argparse
import re
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
SPEC = ROOT / "docs/specs/stdlib/modules/appkit.md"
OUT = ROOT / "stdlib/Objective_C/AppKit/generated.jai"
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
    if cls == "NSView" and name == "convertPoint":
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


def generate(output):
    spec = SPEC.read_text()
    out = "// Generated from reviewed appkit.md by generate_appkit.py.\n\n"
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
libobjc :: #library,system "objc";
CoreGraphics :: #library,system "CoreGraphics";
binding_class :: (name: *u8) -> *void #foreign libobjc "objc_getClass";
binding_selector :: (name: *u8) -> O.Selector #foreign libobjc "sel_registerName";
binding_send :: () #foreign libobjc "objc_msgSend";
#if CPU == .X64 binding_send_stret :: () #foreign libobjc "objc_msgSend_stret";
"""
    output.write_text(out)


def public_inventory():
    """Describe every emitted public declaration, without equating source with native proof."""
    native = {
        "NSView.addSubView",
        "NSView.frame",
        "NSView.bounds",
        "NSView.setFrame",
        "NSView.convertPointToBacking",
        "NSView.convertPointFromBacking",
        "NSView.convertRectToBacking",
        "NSView.convertRectFromBacking",
        "NSView.mouse_inRect",
        "NSView.isInFullScreenMode",
        "NSEvent.type",
        "NSEvent.characters",
        "NSEvent.charactersIgnoringModifiers",
        "NSEvent.keyCode",
        "NSEvent.modifierFlags",
        "NSEvent.isARepeat",
        "NSOpenGLContext.currentContext",
        "NSApp",
        "NSDefaultRunLoopMode",
        "init_app_kit",
        "init_core_graphics_runtime_constants",
        "CGWindowLevelForKey",
    }
    boundaries = {
        "NSView.convertPoint",
        "NSEvent.keyRepeatDelay",
        "NSEvent.keyRepeatInterval",
        "NSEvent.pressedMouseButtons",
    }
    rows = []
    for path in (OUT.parent / "module.jai", OUT):
        owner = None
        enum_owner = None
        lines = path.read_text().split("#scope_file", 1)[0].splitlines()
        for index, line in enumerate(lines):
            nominal = re.match(r"(?:using )?(\w+) :: (struct|enum(?:_flags)?)\b", line)
            if nominal:
                name, kind = nominal.groups()
                rows.append(
                    dict(
                        api=name,
                        kind=kind,
                        body="implemented",
                        contract="reviewed",
                        verification="source-layout",
                    )
                )
                owner = name if kind == "struct" else None
                enum_owner = name if kind.startswith("enum") else None
            elif line == "}" or line.startswith("} @"):
                owner = enum_owner = None
            elif owner:
                field = re.match(r"    ((?:using )?(?:#as )?)(\w+): (.*);$", line)
                if field:
                    rows.append(
                        dict(
                            api=owner + "." + field[2],
                            kind="field",
                            declaration=field[1] + field[2] + ": " + field[3],
                            body="implemented",
                            contract="reviewed",
                            verification="public-source-layout; no full SDK instance size claim",
                        )
                    )
                    continue
                member = re.match(r"    (\w+) ::", line)
                if not member:
                    continue
                name = owner + "." + member[1]
                evidence = "native-positive" if name in native else "source-only"
                if name in boundaries:
                    evidence = "reviewed-receiver-boundary; source-only"
                if (owner, member[1]) in VOID_RESULT:
                    evidence = "native-nil-receiver; nonnil-effect-unverified"
                rows.append(
                    dict(
                        api=name,
                        kind="method",
                        body="implemented",
                        contract="reviewed",
                        verification=evidence,
                        signature=" ".join(
                            "\n".join(lines[index:]).split("{", 1)[0].split()
                        )
                        if "(" in line
                        else line.strip(),
                    )
                )
            elif enum_owner:
                member = re.match(r"    (\w+) ::", line)
                if member:
                    rows.append(
                        dict(
                            api=member[1],
                            kind="enum-member",
                            body="implemented",
                            contract="reviewed",
                            verification="source-value",
                        )
                    )
            else:
                global_ = re.match(r"(\w+)\s*:", line)
                if not global_:
                    continue
                name = global_[1]
                evidence = (
                    "native-positive"
                    if name in native or re.match(r"kCG.*Level$", name)
                    else "source-only"
                )
                rows.append(
                    dict(
                        api=name,
                        kind="declaration",
                        body="implemented",
                        contract="reviewed",
                        verification=evidence,
                    )
                )
    return rows


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--output", type=Path, default=OUT)
    generate(parser.parse_args().output)
