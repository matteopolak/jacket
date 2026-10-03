#if OS == .WINDOWS {
    get_anon_struct_Zone :: Fields.get_anon_struct_Zone;
    set_anon_struct_Zone :: Fields.set_anon_struct_Zone;
    get_anon_struct_Level :: Fields.get_anon_struct_Level;
    set_anon_struct_Level :: Fields.set_anon_struct_Level;
}

#scope_file
#if OS == .WINDOWS Fields :: #import "internal/net/socket-fields-windows";
