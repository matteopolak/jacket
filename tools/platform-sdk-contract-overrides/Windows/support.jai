// Independent scalar and COM wrappers; no reference implementation bodies.
SUCCEEDED :: (hr: HRESULT) -> bool { return hr >= 0; }
FAILED :: (hr: HRESULT) -> bool { return hr < 0; }
HasOverlappedIoCompleted :: (overlapped: OVERLAPPED) -> bool {
    return overlapped.Internal != STATUS_PENDING;
}
MAKELANGID :: (usPrimaryLanguage: u16, usSubLanguage: u16) -> u16 {
    return (usSubLanguage << 10) | usPrimaryLanguage;
}
IUnknown_QueryInterface :: (this: *IUnknown, riid: *IID, ppvObject: **void) -> HRESULT {
    methods := this.vtable;
    return methods.QueryInterface(this, riid, ppvObject);
}
IUnknown_AddRef :: (this: *IUnknown) -> u32 {
    methods := this.vtable;
    return methods.AddRef(this);
}
IUnknown_Release :: (this: *IUnknown) -> u32 {
    methods := this.vtable;
    return methods.Release(this);
}
vtable :: (obj: *IUnknown) -> *IUnknown_VTable { return obj.vtable; }
safe_release :: (obj: *$T/interface IUnknown) {
    if obj == null return;
    IUnknown_Release(cast(*IUnknown) obj);
}
safe_release_and_reset :: (b: **$T/interface IUnknown) {
    if b == null return;
    safe_release(b.*);
    b.* = null;
}

uid :: (str: string) -> GUID {
    text := str;
    if text.count == 38 {
        Basic.assert(text.data[0] == 123 && text.data[37] == 125);
        text.data += 1;
        text.count = 36;
    }
    Basic.assert(text.count == 36);
    Basic.assert(text.data[8] == 45 && text.data[13] == 45 && text.data[18] == 45 && text.data[23] == 45);
    result: GUID;
    result.Data1 = read_guid_hex(text, 0, 8);
    result.Data2 = cast(u16) read_guid_hex(text, 9, 4);
    result.Data3 = cast(u16) read_guid_hex(text, 14, 4);
    result.Data4[0] = cast(u8) read_guid_hex(text, 19, 2);
    result.Data4[1] = cast(u8) read_guid_hex(text, 21, 2);
    index: s64 = 2;
    while index < 8 {
        result.Data4[index] = cast(u8) read_guid_hex(text, 24 + (index - 2) * 2, 2);
        index += 1;
    }
    return result;
}

#scope_file
Basic :: #import "Basic";
read_guid_hex :: (text: string, start: s64, length: s64) -> u32 {
    result: u32 = 0;
    index: s64 = 0;
    while index < length {
        character := text.data[start + index];
        value: u32;
        if character >= 48 && character <= 57 value = character - 48;
        else if character >= 65 && character <= 70 value = character - 55;
        else if character >= 97 && character <= 102 value = character - 87;
        else {
            Basic.assert(false, "Invalid GUID hexadecimal digit");
            return 0;
        }
        result = (result << 4) | value;
        index += 1;
    }
    return result;
}

assert :: (arg: bool, loc := #caller_location) {
    Basic.assert(arg, loc = loc);
}
