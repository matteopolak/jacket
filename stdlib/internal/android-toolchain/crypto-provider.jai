// Canonical opaque native types. Their storage belongs exclusively to EVP.
#scope_module
evp_pkey_st :: struct {}
EVP_MD_CTX :: struct {}
EVP_PKEY_CTX :: struct {}
EVP_MD :: struct {}
ENGINE :: struct {}

EVP_PKEY_free :: (key: *evp_pkey_st) #foreign android_crypto;
EVP_PKEY_is_a :: (key: *evp_pkey_st, name: *u8) -> s32 #foreign android_crypto;
EVP_MD_CTX_new :: () -> *EVP_MD_CTX #foreign android_crypto;
EVP_MD_CTX_free :: (ctx: *EVP_MD_CTX) #foreign android_crypto;
EVP_sha256 :: () -> *EVP_MD #foreign android_crypto;
EVP_DigestSignInit :: (ctx: *EVP_MD_CTX, key_context: **EVP_PKEY_CTX, digest: *EVP_MD, engine: *ENGINE, key: *evp_pkey_st) -> s32 #foreign android_crypto;
EVP_DigestSignUpdate :: (ctx: *EVP_MD_CTX, bytes: *void, count: u64) -> s32 #foreign android_crypto;
EVP_DigestSignFinal :: (ctx: *EVP_MD_CTX, signature: *u8, count: *u64) -> s32 #foreign android_crypto;
EVP_DigestVerifyInit :: (ctx: *EVP_MD_CTX, key_context: **EVP_PKEY_CTX, digest: *EVP_MD, engine: *ENGINE, key: *evp_pkey_st) -> s32 #foreign android_crypto;
EVP_DigestVerifyUpdate :: (ctx: *EVP_MD_CTX, bytes: *void, count: u64) -> s32 #foreign android_crypto;
EVP_DigestVerifyFinal :: (ctx: *EVP_MD_CTX, signature: *u8, count: u64) -> s32 #foreign android_crypto;

#scope_export
libressl_get_public_key :: (subject_public_key_info: string) -> *evp_pkey_st {
    if !crypto_der_length(subject_public_key_info) return null;

    cursor := subject_public_key_info.data;
    key := d2i_PUBKEY(null, *cursor, cast(Crypto_Long) subject_public_key_info.count);

    if !crypto_check(ifx key != null then 1 else 0, #caller_location) return null;

    if cursor != subject_public_key_info.data + subject_public_key_info.count {
        EVP_PKEY_free(key);
        crypto_check(0, #caller_location);
        return null;
    }

    return key;
}

libressl_get_private_key :: (private_key_info: string) -> *evp_pkey_st {
    if !crypto_der_length(private_key_info) return null;

    cursor := private_key_info.data;
    key := d2i_AutoPrivateKey(null, *cursor, cast(Crypto_Long) private_key_info.count);

    if !crypto_check(ifx key != null then 1 else 0, #caller_location) return null;

    if cursor != private_key_info.data + private_key_info.count {
        EVP_PKEY_free(key);
        crypto_check(0, #caller_location);
        return null;
    }

    return key;
}

CheckSSL :: (result: s32, loc := #caller_location) #expand {
    crypto_check(result, loc);
}

libressl_error_callback :: (str: *u8, len: u64, u: *void) -> s32 #c_call {
    // ERR_print_errors_cb invokes synchronously; userdata outlives the drain.
    supplied_context := cast(*#Context) u;
    push_context supplied_context.* {
        context.logger(.{ cast(s64) len, str }, context.logger_data, Log_Info.{ common_flags = .ERROR });
    }

    return 1;
}

sign_data :: (private_key_info: string, message: string) -> bool, string {
    key := libressl_get_private_key(private_key_info);
    if !key return false, "";

    defer EVP_PKEY_free(key);
    digest_context := EVP_MD_CTX_new();
    if !crypto_check(ifx digest_context != null then 1 else 0, #caller_location) return false, "";

    defer EVP_MD_CTX_free(digest_context);

    if !crypto_check(EVP_DigestSignInit(digest_context, null, EVP_sha256(), null, key), #caller_location) return false, "";

    if message.count < 0 {
        crypto_check(0, #caller_location);
        return false, "";
    }

    if !crypto_check(EVP_DigestSignUpdate(digest_context, message.data, cast(u64) message.count), #caller_location) return false, "";

    count: u64;
    if !crypto_check(EVP_DigestSignFinal(digest_context, null, *count), #caller_location) return false, "";

    if count == 0 || count > 0x7fffffffffffffff {
        crypto_check(0, #caller_location);
        return false, "";
    }

    signature := alloc_string(cast(s64) count);
    capacity := count;

    if !crypto_check(EVP_DigestSignFinal(digest_context, signature.data, *count), #caller_location) {
        free(signature);
        return false, "";
    }

    if count > capacity {
        free(signature);
        crypto_check(0, #caller_location);
        return false, "";
    }

    signature.count = cast(s64) count;

    return true, signature;
}

#scope_module
crypto_check :: (result: s32, loc: Source_Code_Location) -> bool {
    if result == 1 return true;

    context.logger("Native crypto operation failed\n", context.logger_data,
                   Log_Info.{ common_flags = .ERROR, location = loc });
    ERR_print_errors_cb(libressl_error_callback, *context);
    assert(false, "Native crypto operation failed");

    return false;
}

#scope_file
android_crypto :: #library "android-crypto/crypto";
#if OS == .WINDOWS Crypto_Long :: s32;
else Crypto_Long :: s64;

Crypto_Error_Callback :: #type (str: *u8, len: u64, u: *void) -> s32 #c_call;
ERR_print_errors_cb :: (callback: Crypto_Error_Callback, user: *void) #foreign android_crypto;
d2i_PUBKEY :: (key: **evp_pkey_st, cursor: **u8, count: Crypto_Long) -> *evp_pkey_st #foreign android_crypto;
d2i_AutoPrivateKey :: (key: **evp_pkey_st, cursor: **u8, count: Crypto_Long) -> *evp_pkey_st #foreign android_crypto;

crypto_der_length :: (data: string) -> bool {
    maximum: s64 = 0x7fffffffffffffff;

    #if OS == .WINDOWS maximum = 0x7fffffff;

    if data.count <= 0 || data.count > maximum || data.data == null {
        crypto_check(0, #caller_location);
        return false;
    }

    return true;
}
