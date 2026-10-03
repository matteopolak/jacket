Adb_Version_Info :: struct {
    tool_version: string;
    protocol_version: string;
    exe_path: string;
}

Adb_Device :: struct {
    Connection_State :: enum {
        CONNECTING :: 0;
        AUTHORIZING :: 1;
        UNAUTHORIZED :: 2;
        NOPERM :: 3;
        OFFLINE :: 4;
        BOOTLOADER :: 5;
        DEVICE :: 6;
        HOST :: 7;
        RECOVERY :: 8;
        SIDELOAD :: 9;
        UNKNOWN :: 10;
        READY :: DEVICE;
    }
    serial: string;
    connection_state: Connection_State;
    product_name: string;
    model_name: string;
    device_name: string;
    transport_id: int;
}

Adb_Compression_Algorithm :: enum {
    UNSPECIFIED :: 0;
    ANY :: 1;
    NONE :: 2;
    BROTLI :: 3;
    LZ4 :: 4;
    ZSTD :: 5;
}

// Stored in this single public module instance, used directly by all operations.
__adb_path: string;
adb_device_serial: string;
adb_be_verbose := !true;
