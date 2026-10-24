// Errors: what went wrong, from the client (`kind`) and, for errors the server sent, every field
// of its ErrorResponse.

Error_Kind :: enum u8 {
    NONE;
    CONFIG; // a bad URL or setting, or a call made outside `Io.io_run`
    IO; // the connection failed; `io` says how
    TIMED_OUT; // an `Io.with_deadline` limit passed
    CANCELED; // the fiber was canceled
    CLOSED; // the connection was closed, or broken by an earlier failure
    TLS; // the TLS handshake or a TLS record failed
    AUTH; // authentication was refused or could not be done safely
    PROTOCOL; // the server sent something this client does not understand
    SERVER; // an ErrorResponse: `code`, `message` and the rest are set
    DECODE; // a value did not decode into the type asked for
    USAGE; // the call does not fit the result (a missing column, more rows than one, say)
    NOT_FOUND; // `query_one` matched no row
    HOST_REJECTED; // a server answered, but is not the kind `target_session_attrs` asks for
    ABORTED; // a pipelined statement was skipped because an earlier one of its batch failed
}

// The five-character SQLSTATE, packed into a number so that it compares and switches like an
// enum: `if error.code == .UNIQUE_VIOLATION`. Codes not listed here still compare and print
// (`sqlstate_text`). Its class is the first two characters: `sqlstate_class(code)`.
Sqlstate :: enum u64 {
    NONE :: 0;
    SUCCESSFUL_COMPLETION :: 0x3030303030; // 00000
    WARNING :: 0x3030303130; // 01000
    NO_DATA :: 0x3030303230; // 02000
    CONNECTION_EXCEPTION :: 0x3030303830; // 08000
    CONNECTION_DOES_NOT_EXIST :: 0x3330303830; // 08003
    CONNECTION_FAILURE :: 0x3630303830; // 08006
    PROTOCOL_VIOLATION :: 0x3130503830; // 08P01
    FEATURE_NOT_SUPPORTED :: 0x3030304130; // 0A000
    CARDINALITY_VIOLATION :: 0x3030303132; // 21000
    DATA_EXCEPTION :: 0x3030303232; // 22000
    STRING_DATA_RIGHT_TRUNCATION :: 0x3130303232; // 22001
    NUMERIC_VALUE_OUT_OF_RANGE :: 0x3330303232; // 22003
    NULL_VALUE_NOT_ALLOWED :: 0x3430303232; // 22004
    INVALID_DATETIME_FORMAT :: 0x3730303232; // 22007
    DATETIME_FIELD_OVERFLOW :: 0x3830303232; // 22008
    DIVISION_BY_ZERO :: 0x3231303232; // 22012
    CHARACTER_NOT_IN_REPERTOIRE :: 0x3132303232; // 22021
    INVALID_PARAMETER_VALUE :: 0x3332303232; // 22023
    INVALID_TEXT_REPRESENTATION :: 0x3230503232; // 22P02
    INVALID_BINARY_REPRESENTATION :: 0x3330503232; // 22P03
    INVALID_JSON_TEXT :: 0x3530503232; // 22P05
    INTEGRITY_CONSTRAINT_VIOLATION :: 0x3030303332; // 23000
    RESTRICT_VIOLATION :: 0x3130303332; // 23001
    NOT_NULL_VIOLATION :: 0x3230353332; // 23502
    FOREIGN_KEY_VIOLATION :: 0x3330353332; // 23503
    UNIQUE_VIOLATION :: 0x3530353332; // 23505
    CHECK_VIOLATION :: 0x3431353332; // 23514
    EXCLUSION_VIOLATION :: 0x3130503332; // 23P01
    INVALID_CURSOR_STATE :: 0x3030303432; // 24000
    INVALID_TRANSACTION_STATE :: 0x3030303532; // 25000
    ACTIVE_SQL_TRANSACTION :: 0x3130303532; // 25001
    READ_ONLY_SQL_TRANSACTION :: 0x3630303532; // 25006
    IN_FAILED_SQL_TRANSACTION :: 0x3230503532; // 25P02
    IDLE_IN_TRANSACTION_SESSION_TIMEOUT :: 0x3330503532; // 25P03
    INVALID_SQL_STATEMENT_NAME :: 0x3030303632; // 26000
    INVALID_AUTHORIZATION_SPECIFICATION :: 0x3030303832; // 28000
    INVALID_PASSWORD :: 0x3130503832; // 28P01
    TRANSACTION_ROLLBACK :: 0x3030303034; // 40000
    SERIALIZATION_FAILURE :: 0x3130303034; // 40001
    STATEMENT_COMPLETION_UNKNOWN :: 0x3330303034; // 40003
    DEADLOCK_DETECTED :: 0x3130503034; // 40P01
    SYNTAX_ERROR_OR_ACCESS_RULE_VIOLATION :: 0x3030303234; // 42000
    SYNTAX_ERROR :: 0x3130363234; // 42601
    INSUFFICIENT_PRIVILEGE :: 0x3130353234; // 42501
    CANNOT_COERCE :: 0x3634383234; // 42846
    GROUPING_ERROR :: 0x3330383234; // 42803
    DATATYPE_MISMATCH :: 0x3430383234; // 42804
    WRONG_OBJECT_TYPE :: 0x3930383234; // 42809
    UNDEFINED_COLUMN :: 0x3330373234; // 42703
    UNDEFINED_FUNCTION :: 0x3338383234; // 42883
    UNDEFINED_TABLE :: 0x3130503234; // 42P01
    UNDEFINED_PARAMETER :: 0x3230503234; // 42P02
    UNDEFINED_OBJECT :: 0x3430373234; // 42704
    DUPLICATE_COLUMN :: 0x3130373234; // 42701
    DUPLICATE_DATABASE :: 0x3430503234; // 42P04
    DUPLICATE_FUNCTION :: 0x3332373234; // 42723
    DUPLICATE_PREPARED_STATEMENT :: 0x3530503234; // 42P05
    DUPLICATE_SCHEMA :: 0x3630503234; // 42P06
    DUPLICATE_TABLE :: 0x3730503234; // 42P07
    DUPLICATE_OBJECT :: 0x3031373234; // 42710
    AMBIGUOUS_COLUMN :: 0x3230373234; // 42702
    AMBIGUOUS_FUNCTION :: 0x3532373234; // 42725
    INDETERMINATE_DATATYPE :: 0x3831503234; // 42P18
    INVALID_COLUMN_REFERENCE :: 0x3031503234; // 42P10
    INVALID_OBJECT_DEFINITION :: 0x3731503234; // 42P17
    INSUFFICIENT_RESOURCES :: 0x3030303335; // 53000
    DISK_FULL :: 0x3030313335; // 53100
    OUT_OF_MEMORY :: 0x3030323335; // 53200
    TOO_MANY_CONNECTIONS :: 0x3030333335; // 53300
    PROGRAM_LIMIT_EXCEEDED :: 0x3030303435; // 54000
    OBJECT_NOT_IN_PREREQUISITE_STATE :: 0x3030303535; // 55000
    OBJECT_IN_USE :: 0x3630303535; // 55006
    LOCK_NOT_AVAILABLE :: 0x3330503535; // 55P03
    OPERATOR_INTERVENTION :: 0x3030303735; // 57000
    QUERY_CANCELED :: 0x3431303735; // 57014
    ADMIN_SHUTDOWN :: 0x3130503735; // 57P01
    CRASH_SHUTDOWN :: 0x3230503735; // 57P02
    CANNOT_CONNECT_NOW :: 0x3330503735; // 57P03
    DATABASE_DROPPED :: 0x3430503735; // 57P04
    IDLE_SESSION_TIMEOUT :: 0x3530503735; // 57P05
    SYSTEM_ERROR :: 0x3030303835; // 58000
    RAISE_EXCEPTION :: 0x3130303050; // P0001
    NO_DATA_FOUND :: 0x3230303050; // P0002
    TOO_MANY_ROWS :: 0x3330303050; // P0003
    ASSERT_FAILURE :: 0x3430303050; // P0004
    INTERNAL_ERROR :: 0x3030305858; // XX000
    DATA_CORRUPTED :: 0x3130305858; // XX001
}

Pg_Error :: struct {
    kind: Error_Kind;
    io: Io.Io_Error; // IO, TIMED_OUT, CANCELED: the Io error behind it

    code: Sqlstate; // SERVER
    severity: string; // "ERROR", "FATAL", "PANIC" (localized when the server is)
    message: string; // the primary message; for client errors, what happened
    detail: string;
    hint: string;
    position: s32; // 1-based character in the query the error points at, or 0
    internal_position: s32;
    internal_query: string;
    where: string; // the call stack of a PL/pgSQL function, say
    schema: string;
    table: string;
    column: string;
    data_type: string;
    constraint: string;
    file: string; // the server's source location
    line: string;
    routine: string;

    // Every string above points into this, which belongs to the error: release it with
    // `free_error` (empty for errors that only hold literals).
    storage: [] u8;
}

// One line describing the error, in temporary storage.
error_text :: (error: Pg_Error) -> string {
    if error.kind == .NONE return "no error";

    builder: String_Builder;
    builder.allocator = temp;

    if error.kind == .SERVER {
        print_to_builder(*builder, "% %: %", ifx error.severity else "ERROR", sqlstate_text(error.code), error.message);
        if error.detail print_to_builder(*builder, " (detail: %)", error.detail);
        if error.hint print_to_builder(*builder, " (hint: %)", error.hint);
    } else {
        print_to_builder(*builder, "%: %", kind_text(error.kind), error.message);

        if error.io != .NONE && error.kind != .TIMED_OUT && error.kind != .CANCELED {
            print_to_builder(*builder, " (%)", error.io);
        }
    }

    return builder_to_string(*builder,, temp);
}

free_error :: (error: *Pg_Error) {
    if error.storage.data free(error.storage.data);
    error.* = .{};
}

// The five characters of a code, in temporary storage.
sqlstate_text :: (code: Sqlstate) -> string {
    text := talloc_string(SQLSTATE_LENGTH);
    for 0..SQLSTATE_LENGTH - 1 text[it] = cast,trunc(u8) ((cast(u64) code) >> (8 * it));
    return text;
}

// The code with only its class kept: `sqlstate_class(.UNIQUE_VIOLATION) == .INTEGRITY_CONSTRAINT_VIOLATION`.
sqlstate_class :: (code: Sqlstate) -> Sqlstate {
    return cast(Sqlstate) ((cast(u64) code & 0xffff) | (0x3030303030 & ~cast(u64) 0xffff));
}

SQLSTATE_LENGTH :: 5;

sqlstate_of :: (text: string) -> Sqlstate {
    if text.count != SQLSTATE_LENGTH return .NONE;
    value: u64;
    for text value |= (cast(u64) it) << (8 * it_index);
    return cast(Sqlstate) value;
}

#scope_module

kind_text :: (kind: Error_Kind) -> string {
    if kind == {
        case .CONFIG; return "configuration error";
        case .IO; return "connection error";
        case .TIMED_OUT; return "timed out";
        case .CANCELED; return "canceled";
        case .CLOSED; return "connection closed";
        case .TLS; return "TLS error";
        case .AUTH; return "authentication failed";
        case .PROTOCOL; return "protocol error";
        case .DECODE; return "decode error";
        case .USAGE; return "usage error";
        case .NOT_FOUND; return "not found";
        case .HOST_REJECTED; return "no suitable server";
        case .ABORTED; return "aborted";
    }

    return "error";
}

// A client-side error. `message` is copied into the error's own storage.
make_error :: (kind: Error_Kind, message: string, io := Io.Io_Error.NONE) -> Pg_Error {
    error: Pg_Error;
    error.kind = kind;
    error.io = io;
    error.storage = cast([] u8) copy_string(message);
    error.message = cast(string) error.storage;

    return error;
}

make_error :: (kind: Error_Kind, format: string, args: ..Any, io := Io.Io_Error.NONE) -> Pg_Error {
    return make_error(kind, tprint(format, ..args), io);
}

io_failure :: (io: Io.Io_Error, what: string) -> Pg_Error {
    kind := Error_Kind.IO;
    if io == .TIMED_OUT kind = .TIMED_OUT;
    if io == .CANCELED kind = .CANCELED;
    return make_error(kind, what, io);
}

// An ErrorResponse or NoticeResponse body: fields of a type byte and a C string, ending in a zero.
// The body is copied, and the strings point into the copy.
parse_server_error :: (body: [] u8) -> Pg_Error {
    error: Pg_Error;
    error.kind = .SERVER;
    error.storage = NewArray(body.count, u8, initialized = false);
    memcpy(error.storage.data, body.data, body.count);
    at := 0;
    while at < error.storage.count {
        field := error.storage[at];
        if field == 0 break;

        at += 1;
        start := at;

        while at < error.storage.count && error.storage[at] != 0 at += 1;

        value := string.{ at - start, error.storage.data + start };

        at += 1;

        if field == {
            case #char "V"; error.severity = value; // never localized
            case #char "S"; if !error.severity error.severity = value;
            case #char "C"; error.code = sqlstate_of(value);
            case #char "M"; error.message = value;
            case #char "D"; error.detail = value;
            case #char "H"; error.hint = value;
            case #char "P"; error.position = cast(s32) string_to_int(value);
            case #char "p"; error.internal_position = cast(s32) string_to_int(value);
            case #char "q"; error.internal_query = value;
            case #char "W"; error.where = value;
            case #char "s"; error.schema = value;
            case #char "t"; error.table = value;
            case #char "c"; error.column = value;
            case #char "d"; error.data_type = value;
            case #char "n"; error.constraint = value;
            case #char "F"; error.file = value;
            case #char "L"; error.line = value;
            case #char "R"; error.routine = value;
        }
    }

    return error;
}
