// Android wire ABI: official androidfw/ResourceTypes.h.
ResStringPool_ref :: struct {
    index: u32;
}

Res_value :: struct {
    size: u16;
    res0: u8;
    dataType: u8;
    data: u32;
}

ResXMLTree_attribute :: struct {
    ns: ResStringPool_ref;
    name: ResStringPool_ref;
    rawValue: ResStringPool_ref;
    typedValue: Res_value;
}

Android_Install_Location :: enum u32 {
    AUTO :: 0;
    INTERNAL_ONLY :: 1;
    PREFER_EXTERNAL :: 2;
}

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
