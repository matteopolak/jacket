#if OS == .LINUX || OS == .ANDROID {
    get_anon_struct_1_ack :: Fields.get_anon_struct_1_ack;
    set_anon_struct_1_ack :: Fields.set_anon_struct_1_ack;
    get_anon_struct_1_doff :: Fields.get_anon_struct_1_doff;
    set_anon_struct_1_doff :: Fields.set_anon_struct_1_doff;
    get_anon_struct_1_fin :: Fields.get_anon_struct_1_fin;
    set_anon_struct_1_fin :: Fields.set_anon_struct_1_fin;
    get_anon_struct_1_psh :: Fields.get_anon_struct_1_psh;
    set_anon_struct_1_psh :: Fields.set_anon_struct_1_psh;
    get_anon_struct_1_res1 :: Fields.get_anon_struct_1_res1;
    set_anon_struct_1_res1 :: Fields.set_anon_struct_1_res1;
    get_anon_struct_1_res2 :: Fields.get_anon_struct_1_res2;
    set_anon_struct_1_res2 :: Fields.set_anon_struct_1_res2;
    get_anon_struct_1_rst :: Fields.get_anon_struct_1_rst;
    set_anon_struct_1_rst :: Fields.set_anon_struct_1_rst;
    get_anon_struct_1_syn :: Fields.get_anon_struct_1_syn;
    set_anon_struct_1_syn :: Fields.set_anon_struct_1_syn;
    get_anon_struct_1_urg :: Fields.get_anon_struct_1_urg;
    set_anon_struct_1_urg :: Fields.set_anon_struct_1_urg;
    get_anon_struct_th_off :: Fields.get_anon_struct_th_off;
    set_anon_struct_th_off :: Fields.set_anon_struct_th_off;
    get_anon_struct_th_x2 :: Fields.get_anon_struct_th_x2;
    set_anon_struct_th_x2 :: Fields.set_anon_struct_th_x2;
}

#if OS == .LINUX {
    get_tcpi_rcv_wscale :: Fields.get_tcpi_rcv_wscale;
    set_tcpi_rcv_wscale :: Fields.set_tcpi_rcv_wscale;
    get_tcpi_snd_wscale :: Fields.get_tcpi_snd_wscale;
    set_tcpi_snd_wscale :: Fields.set_tcpi_snd_wscale;
}

#scope_file
#if OS == .LINUX || OS == .ANDROID Fields :: #import "internal/net/socket-fields-tcp-unix";
