// Is an index into a binary XML string pool, in the layout of Android's resource format.
ResStringPool_ref :: struct {
    index: u32;
}

// Is a typed attribute value in binary XML; the layout is the `Res_value` of Android's resource format.
//
// `dataType` is the type code and `data` the value; its meaning depends on the type.
Res_value :: struct {
    size: u16;
    res0: u8;
    dataType: u8;
    data: u32;
}

// Is one attribute of a binary XML element, in the layout of Android's resource format.
//
// `ns`, `name` and `rawValue` are indices into the manifest's string pool; a `rawValue` index of `DUMMY_U32`
// means the value is only in `typedValue`.
ResXMLTree_attribute :: struct {
    ns: ResStringPool_ref;
    name: ResStringPool_ref;
    rawValue: ResStringPool_ref;
    typedValue: Res_value;
}

// Lists the values of the manifest `installLocation` attribute.
Android_Install_Location :: enum u32 {
    AUTO :: 0;
    INTERNAL_ONLY :: 1;
    PREFER_EXTERNAL :: 2;
}

// Lists the values of the manifest `screenOrientation` attribute, with the negative values Android uses for
// unset and unspecified.
Android_Screen_Orientation_ :: enum s32 {
    UNSET :: -2;
    UNSPECIFIED :: -1;
    LANDSCAPE :: 0;
    PORTRAIT :: 1;
    USER :: 2;
    BEHIND :: 3;
    SENSOR :: 4;
    NOSENSOR :: 5;
    SENSOR_LANDSCAPE :: 6;
    SENSOR_PORTRAIT :: 7;
    REVERSE_LANDSCAPE :: 8;
    REVERSE_PORTRAIT :: 9;
    FULL_SENSOR :: 10;
    USER_LANDSCAPE :: 11;
    USER_PORTRAIT :: 12;
    FULL_USER :: 13;
    LOCKED :: 14;
}

LAUNCH_MODE_SINGLE_TASK: u32 : 2;
DUMMY_U32: u32 : 0xffffffff;
ANDROID_XML_SCHEMA :: "http://schemas.android.com/apk/res/android";
USE_FULL_MODULE :: false;
__debug_manifest_reading := false;

// Is a parsed binary `AndroidManifest.xml`.
//
// Read one with [read_binary_manifest_from_memory], [read_binary_manifest_from_file] or
// [read_binary_manifest_from_apk] and release it with [deinit]. `string_pool` holds the document's strings in
// file order, `resource_ids` the resource id of each attribute name, and `elements` the start and end tags in
// document order. `raw_data` is the encoded input; `free_data_on_deinit` is set when the manifest owns it.
Android_Manifest_Binary :: struct {
    raw_data: string;
    free_data_on_deinit := false;
    string_pool: [] string;
    resource_ids: [] u32;
    Xml_Namespace :: struct {
        prefix: string;
        uri: string;
    }
    android_namespace: Xml_Namespace;
    Xml_Element :: struct {
        kind: enum {
            START;
            END;
        }
        name: string;
        attributes: [] ResXMLTree_attribute;
    }
    elements: [..] Xml_Element;
}
