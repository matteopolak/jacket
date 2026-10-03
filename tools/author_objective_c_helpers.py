"""Independently authored bridges for normalized Objective-C API contracts.

These bodies use SDK primitives and the local reflection descriptor contract.
No reference procedure implementation is read by this module.
"""
from __future__ import annotations

RUNTIME = {
    'to_string': '''if str == null return "";
encoded := NSString.dataUsingEncoding(str, NSUTF8StringEncoding, NO);
if encoded == null return "";
length := NSData.length(encoded);
if length > 0x7fff_ffff_ffff_ffff return "";
bytes := cast(*u8) NSData.bytes(encoded);
if length > 0 && bytes == null return "";
return Basic.copy_string(string.{cast(s64) length, bytes});''',
    'init_objective_c_selector_struct': '''info := type_info(T);
if info.type != .STRUCT || selectors == null return;
record := cast(*Type_Info_Struct) info;
for member: record.members {
    if (cast(u32) member.flags & 3) != 0 || member.type != type_info(Selector) continue;
    if member.offset_in_bytes < 0 || member.offset_in_bytes > info.runtime_size - size_of(Selector) continue;
    bytes := Basic.to_c_string(member.name);
    if bytes == null continue;
    for index: 0..member.name.count-1 if bytes[index] == #char "_" bytes[index] = #char ":";
    slot := cast(*Selector) (cast(*u8) selectors + member.offset_in_bytes);
    slot.* = cast(Selector) sel_registerName(bytes);
    Basic.free(bytes);
}''',
    'objc_finalize_class': 'if my_class != null objc_registerClassPair(my_class);',
    'has_isa_member': '''if ty == null return false;
for member: ty.members {
    if (cast(u32) member.flags & 3) != 0 || member.offset_in_bytes != 0 continue;
    if member.name == "isa" && member.type == type_info(Class) return true;
    if (cast(u32) member.flags & 16) != 0 && member.type.type == .STRUCT {
        if has_isa_member(cast(*Type_Info_Struct) member.type) return true;
    }
}
return false;''',
    'objc_msgSend_typed': '''invoke: (sdk_receiver: *void, sdk_selector: SEL) -> *void #c_call;
invoke = cast(type_of(invoke)) objc_msgSend;
return invoke(target, cast(SEL) selector);''',
}

STRING_RUNTIME = {'objc_lookUpClass', 'objc_getClass', 'objc_allocateClassPair',
                  'objc_getProtocol', 'sel_registerName', 'sel_getUid', 'selector'}


def runtime_body(name: str, signature: str):
    if name in RUNTIME:
        return RUNTIME[name]
    if name in STRING_RUNTIME:
        argument = 'sel_str' if name == 'selector' else 'name'
        native = 'sel_registerName' if name == 'selector' else name
        call = ('super, bytes, extra_bytes' if name == 'objc_allocateClassPair' else 'bytes')
        conversion = 'cast(Selector) ' if name in {'sel_registerName', 'sel_getUid', 'selector'} else ''
        return f'''bytes := Basic.to_c_string({argument});
if bytes == null return null;
defer Basic.free(bytes);
return {conversion}{native}({call});'''
    if name == 'class':
        if '$type: Type' not in signature:
            return '''invoke: (sdk_receiver: *void, sdk_selector: SEL) -> Class #c_call;
invoke = cast(type_of(invoke)) objc_msgSend;
return invoke(cast(*void) self, sel_registerName(cast(*u8) "class\\0"));'''
        return '''info := type_info(type);
if info.type != .STRUCT return null;
name := (cast(*Type_Info_Struct) info).name;
// Compatibility views use SDK classes rather than a shipped custom binary.
if name == "LightweightOpenGLView" return objc_getClass(cast(*u8) "NSOpenGLView\\0");
if name == "LightweightRenderingView" return objc_getClass(cast(*u8) "NSView\\0");
if name == "LightweightMetalView" return objc_getClass(cast(*u8) "NSView\\0");
buffer: [1024] u8;
length: s64 = 0;
while length < name.count && name[length] != #char "(" {
    if length >= 1023 return null;
    buffer[length] = name[length];
    length += 1;
}
buffer[length] = 0;
return objc_getClass(*buffer[0]);'''
    if name == 'superclass':
        return '''invoke: (sdk_receiver: *void, sdk_selector: SEL) -> Class #c_call;
invoke = cast(type_of(invoke)) objc_msgSend;
return invoke(cast(*void) self, sel_registerName(cast(*u8) "superclass\\0"));'''
    if name == 'isEqual':
        return '''invoke: (sdk_receiver: *void, sdk_selector: SEL, value: id) -> BOOL #c_call;
invoke = cast(type_of(invoke)) objc_msgSend;
return invoke(cast(*void) self, sel_registerName(cast(*u8) "isEqual:\\0"), object) != 0;'''
    if name in {'objc_alloc', 'objc_new', 'objc_init', 'objc_copy', 'autorelease', 'release', 'retain'}:
        method = {'objc_alloc': 'alloc', 'objc_new': 'new', 'objc_init': 'init', 'objc_copy': 'copy'}.get(name, name)
        receiver = 'cast(*void) self'
        result = 'id'
        if name == 'objc_alloc' and '$type: Type' not in signature:
            receiver = 'cast(*void) class'
        elif '$type: Type' in signature:
            receiver = 'cast(*void) class(type)'
        if name in {'objc_new', 'objc_init'} or ('$type: Type' in signature):
            result = '*type'
        elif name in {'objc_copy', 'retain'}:
            result = '*instancetype'
        if name == 'release':
            return '''invoke: (sdk_receiver: *void, sdk_selector: SEL) -> void #c_call;
invoke = cast(type_of(invoke)) objc_msgSend;
invoke(cast(*void) self, sel_registerName(cast(*u8) "release\\0"));
return null;'''
        return f'''invoke: (sdk_receiver: *void, sdk_selector: SEL) -> {result} #c_call;
invoke = cast(type_of(invoke)) objc_msgSend;
return invoke({receiver}, sel_registerName(cast(*u8) "{method}\\0"));'''
    return None


def helper_body(relative: str, owner: str | None, name: str, signature: str):
    if relative == 'Objective_C/module.jai':
        return runtime_body(name, signature)
    if relative == 'Objective_C/Foundation.jai':
        if name == 'NSMakePoint':
            return 'return NSPoint.{x, y};'
        if name == 'NSMakeSize':
            return 'return NSSize.{w, h};'
        if name == 'NSMakeRect':
            return 'return NSRect.{NSMakePoint(x, y), NSMakeSize(w, h)};'
        if owner == 'NSString' and name == 'initWithString':
            return '''if str.count < 0 return null;
return NSString.initWithBytes(self, str.data, cast(NSUInteger) str.count, NSUTF8StringEncoding);'''
        if owner == 'NSString' and name == 'getTempString':
            return '''value := objc_alloc(NSString);
value = NSString.initWithString(value, str);
return cast(*NSString) autorelease(value);'''
        if owner == 'NSArray' and name == 'objectAtIndex':
            return '''#assert type_info(Object_Type).type == .POINTER;
invoke: (sdk_receiver: *void, sdk_selector: SEL, index: NSUInteger) -> *void #c_call;
invoke = cast(type_of(invoke)) objc_msgSend;
return cast(Object_Type) invoke(cast(*void) self, sel_registerName(cast(*u8) "objectAtIndex:\\0"), index);'''
    if relative == 'Objective_C/AppKit.jai':
        if name in {'setTitle', 'addButtonWithTitle', 'setMessageText', 'setInformativeText'} and 'title: string' in signature:
            tail = 'return null;' if name == 'setTitle' else ''
            return f'''value := NSString.getTempString(title);
{owner}.{name}(self, value);
{tail}'''
        if owner == 'NSCursor' and name == 'set':
            return '''invoke: (sdk_receiver: *void, sdk_selector: SEL) -> void #c_call;
invoke = cast(type_of(invoke)) objc_msgSend;
invoke(cast(*void) cursor, sel_registerName(cast(*u8) "set\\0"));'''
        if owner == 'NSPasteboard' and name == 'readObjectsForClasses':
            return '''invoke: (sdk_receiver: *void, sdk_selector: SEL, classes: *NSArray(Class), options: *NSDictionary(NSPasteboardReadingOptionKey, id)) -> *void #c_call;
invoke = cast(type_of(invoke)) objc_msgSend;
return cast(*NSArray(T)) invoke(cast(*void) self, sel_registerName(cast(*u8) "readObjectsForClasses:options:\\0"), classes, options);'''
    if relative == 'Objective_C/GameController.jai' and name == 'GCController_controllers':
        return '''invoke: (sdk_receiver: *void, sdk_selector: SEL) -> *NSArray(*GCController) #c_call;
invoke = cast(type_of(invoke)) objc_msgSend;
return invoke(cast(*void) objc_getClass(cast(*u8) "GCController\\0"), sel_registerName(cast(*u8) "controllers\\0"));'''
    if relative == 'Objective_C/CoreGraphics.jai' and name == 'init_core_graphics_runtime_constants':
        keys = ['Base', 'Minimum', 'Desktop', 'DesktopIcon', 'BackstopMenu', 'Normal', 'Floating', 'TornOffMenu', 'Dock', 'MainMenu', 'Status', 'ModalPanel', 'PopUpMenu', 'Dragging', 'ScreenSaver', 'Cursor', 'Overlay', 'Help', 'Utility', 'AssistiveTechHigh', 'Maximum']
        return '\n'.join('kCG' + key + 'WindowLevel = CGWindowLevelForKey(kCG' + key + 'WindowLevelKey);' for key in keys)
    if relative == 'Objective_C/LightweightRenderingView/module.jai':
        if name == 'glContext':
            return '''invoke: (sdk_receiver: *void, sdk_selector: SEL) -> *NSOpenGLContext #c_call;
invoke = cast(type_of(invoke)) objc_msgSend;
return invoke(cast(*void) self, sel_registerName(cast(*u8) "openGLContext\\0"));'''
        if name == 'setGlContext':
            return '''invoke: (sdk_receiver: *void, sdk_selector: SEL, value: *NSOpenGLContext) -> void #c_call;
invoke = cast(type_of(invoke)) objc_msgSend;
invoke(cast(*void) self, sel_registerName(cast(*u8) "setOpenGLContext:\\0"), glc);'''
        if name == 'swapBuffers':
            return '''get_context: (sdk_receiver: *void, sdk_selector: SEL) -> *NSOpenGLContext #c_call;
get_context = cast(type_of(get_context)) objc_msgSend;
glc := get_context(self, sel_registerName(cast(*u8) "openGLContext\\0"));
flush: (sdk_receiver: *void, sdk_selector: SEL) -> void #c_call;
flush = cast(type_of(flush)) objc_msgSend;
flush(cast(*void) glc, sel_registerName(cast(*u8) "flushBuffer\\0"));'''
    return None
