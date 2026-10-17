#include <netinet/tcp.h>
#include <stdint.h>
#include <stdio.h>
#include <string.h>
int main(void) {
uint32_t header[4] = {0x54435031,256,sizeof(struct tcphdr),sizeof(struct tcp_connection_info)};
if(fwrite(header,sizeof(header),1,stdout)!=1) return 1;
for(uint32_t i=0;i<256;i++) {
uint32_t pattern=(i*0x01054321u)^0xa5c31f00u;
struct tcphdr packet; memset(&packet,0xa5,sizeof(packet));
packet.th_off=pattern; packet.th_x2=~pattern;
struct tcp_connection_info info; memset(&info,0xa5,sizeof(info));
info.tcpi_tfo_cookie_invalid=(pattern>>8)&1;
info.tcpi_tfo_cookie_rcv=(pattern>>1)&1;
info.tcpi_tfo_cookie_req=(pattern>>0)&1;
info.tcpi_tfo_cookie_req_rcv=(pattern>>6)&1;
info.tcpi_tfo_cookie_sent=(pattern>>7)&1;
info.tcpi_tfo_cookie_wrong=(pattern>>9)&1;
info.tcpi_tfo_heuristics_disable=(pattern>>11)&1;
info.tcpi_tfo_no_cookie_rcv=(pattern>>10)&1;
info.tcpi_tfo_onebyte_proxy=(pattern>>14)&1;
info.tcpi_tfo_recv_blackhole=(pattern>>13)&1;
info.tcpi_tfo_send_blackhole=(pattern>>12)&1;
info.tcpi_tfo_syn_data_acked=(pattern>>4)&1;
info.tcpi_tfo_syn_data_rcv=(pattern>>5)&1;
info.tcpi_tfo_syn_data_sent=(pattern>>3)&1;
info.tcpi_tfo_syn_loss=(pattern>>2)&1;
info.__pad2=pattern;
if(fwrite(&packet,sizeof(packet),1,stdout)!=1) return 2;
if(fwrite(&info,sizeof(info),1,stdout)!=1) return 3;
}
return fflush(stdout)==0?0:4;
}
