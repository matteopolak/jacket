android_generate_jks_keystore :: (java_bin_path: string, keystore_path: string, key_alias := DEFAULT_KEYSTORE_ALIAS, key_password := DEFAULT_KEYSTORE_PASSWORD, algorithm := Android_Signing_Key_Signature_Algorithm.EC, key_size := 0, common_name := "Jai Android", organization_name := "Jai", country_code := "US") -> bool, Android_Signing_Key {
    bits := key_size;
    algorithm_name: string;

    if algorithm == .RSA {
        if bits == 0 bits = 1024;
        if bits != 1024 && bits != 2048 && bits != 4096 && bits != 8192 && bits != 16384 return false, .{};
        algorithm_name = "RSA";
    } else if algorithm == .DSA {
        if bits == 0 bits = 1024;
        if bits != 1024 && bits != 2048 && bits != 3072 return false, .{};
        algorithm_name = "DSA";
    } else if algorithm == .EC {
        if bits != 0 return false, .{};
        algorithm_name = "EC";
    } else return false, .{};

    if java_bin_path.count == 0 || keystore_path.count == 0 || key_alias.count == 0 return false, .{};
    // Do not open/update an existing credential store as a side effect of creation.
    if Jks_Files.file_exists(keystore_path) return false, .{};

    executable := Jks_Strings.join(java_bin_path, "/keytool");

    #if OS == .WINDOWS {
        old := executable;
        executable = Jks_Strings.join(executable, ".exe");
        free(old);
    }

    defer free(executable);
    subject := jks_distinguished_name(common_name, organization_name, country_code);

    defer free(subject);
    size_text := sprint("%", bits);

    defer free(size_text);
    arguments: [..] string;

    defer array_free(arguments);
    for word: string.[executable, "-genkeypair", "-noprompt", "-storetype", "JKS", "-keystore", keystore_path, "-alias", key_alias, "-storepass", key_password, "-keypass", key_password, "-dname", subject, "-validity", "365", "-keyalg", algorithm_name] array_add(*arguments, word);

    if algorithm == .EC {
        for word: string.["-groupname", "secp256r1"] array_add(*arguments, word);
    } else {
        array_add(*arguments, "-keysize");
        array_add(*arguments, size_text);
    }

    result, output, errors, timed_out := Jks_Process.run_command(..arguments, capture_and_return_output = true);

    defer free(output);
    defer free(errors);

    if result.type != .EXITED || result.exit_code != 0 || timed_out {
        context.logger("Java keytool did not create the requested JKS store\n", context.logger_data, Log_Info.{ common_flags = .ERROR });
        return false, .{};
    }

    _, written_size, exists := Jks_Files.file_modtime_and_size(keystore_path);
    if !exists || written_size <= 0 {
        context.logger("Java keytool output is missing or empty\n", context.logger_data, Log_Info.{ common_flags = .ERROR });
        return false, .{};
    }

    key: Android_Signing_Key;
    key.kind = .JKS;
    key.jks.keystore_path = copy_string(keystore_path);
    key.jks.alias = copy_string(key_alias);
    key.jks.password = copy_string(key_password);

    return true, key;
}

#scope_file
using #import "Basic";
Jks_Process :: #import "Process";
Jks_Files :: #import "File_Utilities";
Jks_Strings :: #import "String";

DEFAULT_KEYSTORE_ALIAS :: "jaiiskewl";
DEFAULT_KEYSTORE_PASSWORD :: "password";

jks_distinguished_name :: (common_name: string, organization_name: string, country_code: string) -> string {
    builder: String_Builder;

    defer free_buffers(*builder);
    labels: [3] string = .["CN=", ",O=", ",C="];

    for value, i: string.[common_name, organization_name, country_code] {
        append(*builder, labels[i]);
        append(*builder, "\"");

        for byte: value {
            if byte == 34 || byte == 92 append(*builder, "\\");
            append(*builder, byte);
        }

        append(*builder, "\"");
    }

    return builder_to_string(*builder);
}
