Mail_Server :: struct {
    host: string;
    port: int;
    enable_ssl: bool;
    ssl_ca_bundle_path: string;
    username: string;
    password: string;
    verbose_logging := false;
}

Mail_Message :: struct {
    from: string;
    reply_to: string;
    to: [] string;
    cc: [] string;
    bcc: [] string;
    subject: string;
    body: string;
    attachments: [] string;
    html: bool;
}

mail_send :: (smtp: Mail_Server, msg: Mail_Message) -> bool {
    mark := get_temporary_storage_mark();

    defer set_temporary_storage_mark(mark);
    if !header_value(smtp.host) || !header_value(msg.from) || !header_value(msg.reply_to) || !header_value(msg.subject)
        return failure("Mail header or host contains an unsupported byte\n");
    if has_nul(smtp.username) || has_nul(smtp.password) || has_nul(smtp.ssl_ca_bundle_path)
        return failure("Mail C-string input contains NUL\n");
    if smtp.port < 0 || smtp.port > 65535 || msg.body.count < 0 || msg.to.count + msg.cc.count + msg.bcc.count == 0
        return failure("invalid Mail port, body or recipient list\n");

    R.one_time_init(*global_guard, #code {
        global_status = C.curl_global_init(cast(Native_Long) C.CURL_GLOBAL_DEFAULT);
    });
    if !check(global_status) return false;

    easy := C.curl_easy_init();
    if !easy return failure("curl_easy_init failed\n");

    mime: *C.curl_mime;
    headers: *C.curl_slist;
    recipients: *C.curl_slist;

    defer {
        C.curl_easy_cleanup(easy);
        if mime C.curl_mime_free(mime);
        C.curl_slist_free_all(headers);
        C.curl_slist_free_all(recipients);
    }

    url := sprint("smtp://%", smtp.host);

    defer free(url);
    if !check(C.curl_easy_setopt(easy, .URL, temp_c_string(url))) return false;
    if smtp.port > 0 && !check(C.curl_easy_setopt(easy, .PORT, cast(Native_Long) smtp.port)) return false;
    // Only caller-provided connection/auth fields are used. The library does not
    // load netrc credentials or inherit proxy configuration.
    if !check(C.curl_easy_setopt(easy, .PROXY, temp_c_string(""))) return false;
    if !check(C.curl_easy_setopt(easy, .NETRC, cast(Native_Long) 0)) return false;
    if !check(C.curl_easy_setopt(easy, .NOSIGNAL, cast(Native_Long) 1)) return false;
    if !check(C.curl_easy_setopt(easy, .VERBOSE, cast(Native_Long) smtp.verbose_logging)) return false;

    ssl := C.curl_usessl.NONE;
    if smtp.enable_ssl ssl = .ALL;
    if !check(C.curl_easy_setopt(easy, .USE_SSL, cast(Native_Long) ssl)) return false;
    if smtp.ssl_ca_bundle_path.count > 0 && !check(C.curl_easy_setopt(easy, .CAINFO, temp_c_string(smtp.ssl_ca_bundle_path))) return false;

    if smtp.username.count > 0 || smtp.password.count > 0 {
        if !check(C.curl_easy_setopt(easy, .USERNAME, temp_c_string(smtp.username))) return false;
        if !check(C.curl_easy_setopt(easy, .PASSWORD, temp_c_string(smtp.password))) return false;
    }

    if !check(C.curl_easy_setopt(easy, .MAIL_FROM, temp_c_string(msg.from))) return false;
    if !add_recipients(*recipients, msg.to) || !add_recipients(*recipients, msg.cc) || !add_recipients(*recipients, msg.bcc) return false;
    if !check(C.curl_easy_setopt(easy, .MAIL_RCPT, recipients)) return false;
    if !add_header(*headers, "From", .[msg.from]) || !add_header(*headers, "Subject", .[msg.subject]) return false;
    if msg.reply_to.count > 0 && !add_header(*headers, "Reply-To", .[msg.reply_to]) return false;
    if msg.to.count > 0 && !add_header(*headers, "To", msg.to) return false;
    if msg.cc.count > 0 && !add_header(*headers, "Cc", msg.cc) return false;
    if !check(C.curl_easy_setopt(easy, .HTTPHEADER, headers)) return false;

    mime = C.curl_mime_init(easy);
    if !mime return failure("curl MIME allocation failed\n");

    body := C.curl_mime_addpart(mime);
    if !body return failure("curl MIME body allocation failed\n");
    if !check(C.curl_mime_data(body, msg.body.data, cast(u64) msg.body.count)) return false;

    content_type := "text/plain; charset=utf-8";
    if msg.html content_type = "text/html; charset=utf-8";
    if !check(C.curl_mime_type(body, temp_c_string(content_type))) return false;
    if !check(C.curl_mime_encoder(body, temp_c_string("quoted-printable"))) return false;

    for path: msg.attachments {
        if has_nul(path) return failure("Mail attachment path contains NUL\n");

        part := C.curl_mime_addpart(mime);
        if !part return failure("curl MIME attachment allocation failed\n");
        if !check(C.curl_mime_filedata(part, temp_c_string(path))) return false;
        if !check(C.curl_mime_type(part, temp_c_string("application/octet-stream"))) return false;
        if !check(C.curl_mime_encoder(part, temp_c_string("base64"))) return false;
    }

    if !check(C.curl_easy_setopt(easy, .MIMEPOST, mime)) return false;

    return check(C.curl_easy_perform(easy));
}

#scope_file
#import "Basic";
C :: #import "Curl";
R :: #import "Runtime_Support";

#if OS == .WINDOWS Native_Long :: s32;
else Native_Long :: s64;
global_guard: s32;
global_status: C.CURLcode = .FAILED_INIT;

failure :: (message: string) -> bool {
    context.logger(message, context.logger_data, Log_Info.{ common_flags = .ERROR });
    return false;
}

check :: (code: C.CURLcode) -> bool {
    if code == .OK return true;
    description := C.curl_easy_strerror(code);
    if description return failure(to_string(description));
    return failure("libcurl operation failed\n");
}

has_nul :: (value: string) -> bool {
    for byte: value if byte == 0 return true;
    return false;
}

header_value :: (value: string) -> bool {
    for byte: value if byte == 0 || byte == 10 || byte == 13 return false;
    return true;
}

append_list :: (list: **C.curl_slist, value: string) -> bool {
    next := C.curl_slist_append(list.*, temp_c_string(value));
    if !next return failure("curl string-list allocation failed\n");
    list.* = next;
    return true;
}

add_recipients :: (list: **C.curl_slist, values: [] string) -> bool {
    for value: values {
        if !header_value(value) return failure("Mail recipient contains an unsupported byte\n");
        if !append_list(list, value) return false;
    }

    return true;
}

add_header :: (list: **C.curl_slist, name: string, values: [] string) -> bool {
    builder: String_Builder;

    defer free_buffers(*builder);
    append(*builder, name);
    append(*builder, ": ");

    for value, index: values {
        if !header_value(value) return failure("Mail header contains an unsupported byte\n");
        if index > 0 append(*builder, ", ");
        append(*builder, value);
    }

    if builder.failed return failure("Mail header allocation failed\n");

    line := builder_to_string(*builder);

    defer free(line);

    return append_list(list, line);
}
