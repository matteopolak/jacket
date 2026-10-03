#if OS == .MACOS {
    get_th_off :: Fields.get_th_off;
    set_th_off :: Fields.set_th_off;
    get_th_x2 :: Fields.get_th_x2;
    set_th_x2 :: Fields.set_th_x2;
    get_tcpi_tfo_cookie_invalid :: Fields.get_tcpi_tfo_cookie_invalid;
    set_tcpi_tfo_cookie_invalid :: Fields.set_tcpi_tfo_cookie_invalid;
    get_tcpi_tfo_cookie_rcv :: Fields.get_tcpi_tfo_cookie_rcv;
    set_tcpi_tfo_cookie_rcv :: Fields.set_tcpi_tfo_cookie_rcv;
    get_tcpi_tfo_cookie_req :: Fields.get_tcpi_tfo_cookie_req;
    set_tcpi_tfo_cookie_req :: Fields.set_tcpi_tfo_cookie_req;
    get_tcpi_tfo_cookie_req_rcv :: Fields.get_tcpi_tfo_cookie_req_rcv;
    set_tcpi_tfo_cookie_req_rcv :: Fields.set_tcpi_tfo_cookie_req_rcv;
    get_tcpi_tfo_cookie_sent :: Fields.get_tcpi_tfo_cookie_sent;
    set_tcpi_tfo_cookie_sent :: Fields.set_tcpi_tfo_cookie_sent;
    get_tcpi_tfo_cookie_wrong :: Fields.get_tcpi_tfo_cookie_wrong;
    set_tcpi_tfo_cookie_wrong :: Fields.set_tcpi_tfo_cookie_wrong;
    get_tcpi_tfo_heuristics_disable :: Fields.get_tcpi_tfo_heuristics_disable;
    set_tcpi_tfo_heuristics_disable :: Fields.set_tcpi_tfo_heuristics_disable;
    get_tcpi_tfo_no_cookie_rcv :: Fields.get_tcpi_tfo_no_cookie_rcv;
    set_tcpi_tfo_no_cookie_rcv :: Fields.set_tcpi_tfo_no_cookie_rcv;
    get_tcpi_tfo_onebyte_proxy :: Fields.get_tcpi_tfo_onebyte_proxy;
    set_tcpi_tfo_onebyte_proxy :: Fields.set_tcpi_tfo_onebyte_proxy;
    get_tcpi_tfo_recv_blackhole :: Fields.get_tcpi_tfo_recv_blackhole;
    set_tcpi_tfo_recv_blackhole :: Fields.set_tcpi_tfo_recv_blackhole;
    get_tcpi_tfo_send_blackhole :: Fields.get_tcpi_tfo_send_blackhole;
    set_tcpi_tfo_send_blackhole :: Fields.set_tcpi_tfo_send_blackhole;
    get_tcpi_tfo_syn_data_acked :: Fields.get_tcpi_tfo_syn_data_acked;
    set_tcpi_tfo_syn_data_acked :: Fields.set_tcpi_tfo_syn_data_acked;
    get_tcpi_tfo_syn_data_rcv :: Fields.get_tcpi_tfo_syn_data_rcv;
    set_tcpi_tfo_syn_data_rcv :: Fields.set_tcpi_tfo_syn_data_rcv;
    get_tcpi_tfo_syn_data_sent :: Fields.get_tcpi_tfo_syn_data_sent;
    set_tcpi_tfo_syn_data_sent :: Fields.set_tcpi_tfo_syn_data_sent;
    get_tcpi_tfo_syn_loss :: Fields.get_tcpi_tfo_syn_loss;
    set_tcpi_tfo_syn_loss :: Fields.set_tcpi_tfo_syn_loss;
    get___pad2 :: Fields.get___pad2;
    set___pad2 :: Fields.set___pad2;
}

#scope_file
#if OS == .MACOS Fields :: #import "internal/net/socket-fields-macos";
