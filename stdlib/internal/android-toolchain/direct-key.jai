android_generate_signing_key_direct :: (out_path: string, key_name: string, algorithm := Android_Signing_Key_Signature_Algorithm.EC, key_size := 0, common_name := "Jai Android", organization_name := "Jai", country_code := "US") -> bool, Android_Signing_Key {
    // The generator ignores key_size; actual EC/RSA sizes are fixed.
    if algorithm != .EC && algorithm != .RSA return false, .{};
    if key_name.count == 0 return false, .{};
    for byte: key_name if byte == 0 || byte == 47 || byte == 92 return false, .{};

    #if OS == .MACOS || OS == .LINUX || OS == .ANDROID {
        if !Key_File.make_directory_if_it_does_not_exist(out_path, recursive = true) return false, .{};

        template := Key_String.join(out_path, "/", key_name, "-XXXXXX");

        defer free(template);
        writable := to_c_string(template);

        defer free(writable);
        if !key_mkdtemp(writable) return false, .{};

        directory := copy_string(.{ count = template.count, data = writable });

        defer free(directory);
        certificate := Key_String.join(directory, "/", key_name, ".certificate.der");
        private_key := Key_String.join(directory, "/", key_name, ".private-key.der");
        configuration := Key_String.join(directory, "/openssl.cnf");
        certificate_pem := Key_String.join(directory, "/certificate.pem");
        key_pem := Key_String.join(directory, "/private-key.pem");

        defer free(configuration);
        defer free(certificate_pem);
        defer free(key_pem);
        subject := direct_key_subject(common_name, organization_name, country_code);

        defer free(subject);
        generated := Key_File.write_entire_file(configuration, "[req]\ndistinguished_name=subject\n[subject]\n");
        if generated {
            arguments: [..] string;

            defer array_free(arguments);
            for word: string.["openssl", "req", "-new", "-x509", "-nodes", "-utf8", "-sha256", "-days", "365", "-config", configuration, "-subj", subject, "-keyout", key_pem, "-out", certificate_pem, "-newkey"] array_add(*arguments, word);

            if algorithm == .EC {
                for word: string.["ec", "-pkeyopt", "ec_paramgen_curve:prime256v1"] array_add(*arguments, word);
            } else array_add(*arguments, "rsa:2048");

            generated = direct_key_command(arguments);
        }

        if generated generated = direct_key_command(string.["openssl", "pkcs8", "-topk8", "-nocrypt", "-in", key_pem, "-outform", "DER", "-out", private_key]);
        if generated generated = direct_key_command(string.["openssl", "x509", "-in", certificate_pem, "-outform", "DER", "-out", certificate]);

        if generated {
            private_c := to_c_string(private_key);
            generated = key_chmod(private_c, 0x180) == 0;
            free(private_c);
        }

        cleaned := true;

        for path: string.[configuration, certificate_pem, key_pem] {
            if Key_Utilities.file_exists(path) {
                removed := Key_File.file_delete(path);
                if !removed cleaned = false;
            }
        }

        if generated && cleaned {
            key: Android_Signing_Key;
            key.kind = .ASN1_CERTIFICATE_AND_PRIVATE_KEY;
            key.direct.certificate_path = certificate;
            key.direct.private_key_path = private_key;

            return true, key;
        }

        for path: string.[certificate, private_key] {
            if Key_Utilities.file_exists(path) {
                removed := Key_File.file_delete(path);
                if !removed cleaned = false;
            }
        }

        removed_directory := Key_File.delete_directory(directory);
        if !removed_directory cleaned = false;
        if !cleaned context.logger("Direct signing-key cleanup failed\n", context.logger_data, Log_Info.{ common_flags = .ERROR });

        free(certificate);
        free(private_key);

        return false, .{};
    } else {
        context.logger("Direct signing-key generation needs a supported POSIX host backend\n", context.logger_data, Log_Info.{ common_flags = .ERROR });
        return false, .{};
    }
}

#scope_file
using #import "Basic";
Key_File :: #import "File";
Key_Utilities :: #import "File_Utilities";
Key_String :: #import "String";
Key_Process :: #import "Process";

direct_key_subject :: (common_name: string, organization_name: string, country_code: string) -> string {
    builder: String_Builder;

    defer free_buffers(*builder);
    labels: [3] string = .["/CN=", "/O=", "/C="];

    for value, i: string.[common_name, organization_name, country_code] {
        append(*builder, labels[i]);

        for byte: value {
            if byte == 47 || byte == 92 append(*builder, "\\");
            append(*builder, byte);
        }
    }

    return builder_to_string(*builder);
}

direct_key_command :: (arguments: [] string) -> bool {
    result, output, errors, timed_out := Key_Process.run_command(..arguments, capture_and_return_output = true);

    defer free(output);
    defer free(errors);
    if result.type == .EXITED && result.exit_code == 0 && !timed_out return true;

    context.logger("Native crypto key-generation command failed\n", context.logger_data, Log_Info.{ common_flags = .ERROR });
    if errors.count > 0 context.logger(errors, context.logger_data, Log_Info.{ common_flags = .ERROR });

    return false;
}

#if OS == .MACOS || OS == .LINUX || OS == .ANDROID {
    key_libc :: #system_library "libc";
    key_mkdtemp :: (template: *u8) -> *u8 #foreign key_libc "mkdtemp";
    #if OS == .MACOS Key_Mode :: u16;
    else Key_Mode :: u32;
    key_chmod :: (path: *u8, mode: Key_Mode) -> s32 #foreign key_libc "chmod";
}
