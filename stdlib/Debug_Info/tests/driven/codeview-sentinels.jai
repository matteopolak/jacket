#import "Basic";
D :: #import "Debug_Info";

main :: () {
    assert(type_info(D.Codeview_Leaf_Type).name == "Codeview_Leaf_Type");
    assert(type_info(D.Codeview_Symbol_Type).name == "Codeview_Symbol_Type");
    assert(size_of(D.Codeview_Leaf_Type) == 2 && size_of(D.Codeview_Symbol_Type) == 2);
    assert(cast(u16) D.Codeview_Leaf_Type.ID_LAST == 0x1608);
    assert(cast(u16) D.Codeview_Leaf_Type.ID_MAX == 0x1607);
    assert(cast(u16) D.Codeview_Leaf_Type.TYPE_LAST == 0x151e);
    assert(cast(u16) D.Codeview_Leaf_Type.TYPE_MAX == 0x151d);
    assert(cast(u16) D.Codeview_Symbol_Type.RECTYPE_MAX == 0x1169);
    assert(cast(u16) D.Codeview_Symbol_Type.RECTYPE_LAST == 0x1168);
    assert(cast(u16) D.Codeview_Symbol_Type.RECTYPE_PAD == 0x1269);
    print("ok\n");
}
