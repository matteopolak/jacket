#if OS == .WINDOWS {
    set_icon_by_filename :: Impl.set_icon_by_filename;
    set_icon_by_data :: Impl.set_icon_by_data;
    generate_manifest :: Impl.generate_manifest;
    add_manifest_to_executable :: Impl.add_manifest_to_executable;
    disable_runtime_console :: Impl.disable_runtime_console;
    find_windows_kit_root :: Impl.find_windows_kit_root;
    find_visual_studio_in_a_ridiculous_garbage_way :: Impl.find_visual_studio_in_a_ridiculous_garbage_way;
    ICONDIR :: Impl.ICONDIR;
    ICONDIRENTRY :: Impl.ICONDIRENTRY;
    Manifest_Options :: Impl.Manifest_Options;
}

#scope_file
#if OS == .WINDOWS Impl :: #import "internal/windows-resources";
