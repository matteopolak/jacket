#assert OS_IS_UNIX || OS == .WINDOWS;

#load "native.jai";
#load "fields.jai";
#load "scope-fields.jai";
#load "tcp-unix-fields.jai";
#load "cmsg.jai";

Socket :: Impl.Socket;
INVALID_SOCKET :: Impl.INVALID_SOCKET;
SOCKET_ERROR :: Impl.SOCKET_ERROR;
SOCKET_WOULDBLOCK :: Impl.SOCKET_WOULDBLOCK;
SOCKET_WOULDBLOCK_CONNECT :: Impl.SOCKET_WOULDBLOCK_CONNECT;
socket_init :: Impl.socket_init;
get_last_socket_error :: Impl.get_last_socket_error;
set_blocking :: Impl.set_blocking;
set_keepalive :: Impl.set_keepalive;
to_string :: Impl.to_string;
append_address :: Impl.append_address;
accept :: Impl.accept;
accept_v6 :: Impl.accept_v6;
bind :: Impl.bind;
close_and_reset :: Impl.close_and_reset;

#if OS_IS_UNIX {
    FD_BITS_ELEMENT_TYPE :: Impl.FD_BITS_ELEMENT_TYPE;
    BITS_PER_FD_SET_MASK :: Impl.BITS_PER_FD_SET_MASK;
    FD_ISSET :: Impl.FD_ISSET;
    check_fd_set :: Impl.check_fd_set;
}

#if OS == .WINDOWS {
    IOW :: Impl.IOW;
    FIONBIO :: Impl.FIONBIO;
}

FD_SET :: Impl.FD_SET;
FD_CLR :: Impl.FD_CLR;

#scope_file
Impl :: #import "internal/net/socket";
