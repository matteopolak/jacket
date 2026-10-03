Manifest_Options :: struct {
    dpi_aware := true;
    require_administrator := false;
    use_segment_heap := true;
}

generate_manifest :: (options: Manifest_Options) -> string {
    builder: B.String_Builder;

    defer B.free_buffers(*builder);
    B.append(*builder, "<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"yes\"?>\n<assembly xmlns=\"urn:schemas-microsoft-com:asm.v1\" manifestVersion=\"1.0\">\n");
    B.append(*builder, "<trustInfo xmlns=\"urn:schemas-microsoft-com:asm.v3\"><security><requestedPrivileges><requestedExecutionLevel level=\"");
    if options.require_administrator B.append(*builder, "requireAdministrator");
    else B.append(*builder, "asInvoker");

    B.append(*builder, "\" uiAccess=\"false\"/></requestedPrivileges></security></trustInfo>\n");
    B.append(*builder, "<compatibility xmlns=\"urn:schemas-microsoft-com:compatibility.v1\"><application>\n");
    ids: [5] string = .[
        "{e2011457-1546-43c5-a5fe-008deee3d3f0}",
        "{35138b9a-5d96-4fbd-8e2d-a2440225f93a}",
        "{4a2f28e3-53b9-4441-ba9c-d69d4a4a6e38}",
        "{1f676c76-80e1-4239-95bb-83d0f6d0da78}",
        "{8e0f7a12-bfb3-4fe8-b9a5-48fd50a15a9a}"
    ];
    for id: ids B.print(*builder, "<supportedOS Id=\"%\"/>\n", id);

    B.append(*builder, "</application></compatibility>\n<application xmlns=\"urn:schemas-microsoft-com:asm.v3\"><windowsSettings>\n");

    if options.dpi_aware {
        B.append(*builder, "<dpiAware xmlns=\"http://schemas.microsoft.com/SMI/2005/WindowsSettings\">true/pm</dpiAware>\n");
        B.append(*builder, "<dpiAwareness xmlns=\"http://schemas.microsoft.com/SMI/2016/WindowsSettings\">permonitorv2,permonitor</dpiAwareness>\n");
    } else B.append(*builder, "<dpiAware xmlns=\"http://schemas.microsoft.com/SMI/2005/WindowsSettings\">unaware</dpiAware>\n");

    if options.use_segment_heap B.append(*builder, "<heapType xmlns=\"http://schemas.microsoft.com/SMI/2020/WindowsSettings\">SegmentHeap</heapType>\n");

    B.append(*builder, "<activeCodePage xmlns=\"http://schemas.microsoft.com/SMI/2019/WindowsSettings\">UTF-8</activeCodePage>\n</windowsSettings></application>\n</assembly>\n");

    return B.builder_to_string(*builder);
}

#scope_file
B :: #import "Basic";
