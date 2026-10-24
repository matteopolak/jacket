// SCRAM-SHA-256 and SCRAM-SHA-256-PLUS (RFC 5802, RFC 7677, RFC 5929 tls-server-end-point).

Scram :: struct {
    started: bool;
    verified: bool; // the server proved it knows the password
    plus: bool; // channel binding is in use
    nonce: string;
    client_first_bare: string;
    binding: string; // the gs2 header, then the certificate hash for -PLUS
    server_signature: [32] u8;
}

free_scram :: (s: *Scram) {
    free(s.nonce);
    free(s.client_first_bare);
    free(s.binding);
    s.* = .{};
}

scram_start :: (c: *Connection, config: Config, s: *Scram, mechanisms: [] string) -> Pg_Error {
    offers_plain := array_find(mechanisms, "SCRAM-SHA-256");
    offers_plus := array_find(mechanisms, "SCRAM-SHA-256-PLUS");

    certificate_hash: string;

    #if TLS {
        if c.tls_used && offers_plus && config.channel_binding != .DISABLE {
            certificate_hash = tls_server_end_point(c.tls);
        }
    }

    s.plus = certificate_hash.count > 0;
    if config.channel_binding == .REQUIRE && !s.plus {
        if !c.tls_used return auth_error(c, "channel_binding=require needs TLS");
        if !offers_plus return auth_error(c, "channel_binding=require, but the server does not offer SCRAM-SHA-256-PLUS");
        return auth_error(c, "channel_binding=require, but the server certificate's signature has no hash to bind to");
    }

    if !s.plus && !offers_plain {
        return auth_error(c, tprint("the server offers no SASL mechanism this client supports: %", mechanisms));
    }

    // "y": the client could bind but the server did not offer it, which lets the server notice a
    // downgrade. "n": the client is not binding at all.
    header: string;
    if s.plus header = "p=tls-server-end-point,,";
    else if c.tls_used && !offers_plus && config.channel_binding != .DISABLE header = "y,,";
    else header = "n,,";

    s.binding = join(header, certificate_hash);

    random: [18] u8;
    if !Random.random_bytes(random) return auth_error(c, "the system's random number generator failed");

    s.nonce = Base64.base64_encode(cast(string) random);
    s.client_first_bare = join("n=,r=", s.nonce);
    s.started = true;

    mechanism := ifx s.plus then "SCRAM-SHA-256-PLUS" else "SCRAM-SHA-256";
    first := join(header, s.client_first_bare,, temp);

    begin_message(c, #char "p");
    put_cstring(*c.out, mechanism);
    put_u32(*c.out, cast(u32) first.count);
    put_bytes(*c.out, first);
    end_message(c);

    return send(c);
}

scram_continue :: (c: *Connection, config: Config, s: *Scram, data: [] u8) -> Pg_Error {
    if !s.started return protocol_error(c, "a SASL continuation without a SASL start");

    server_first := cast(string) data;
    server_nonce, salt_text, iteration_text: string;
    rest := server_first;

    while rest {
        found, item, after := split_from_left(rest, #char ",");
        if !found {
            item = rest;
            after = "";
        }

        rest = after;
        if begins_with(item, "r=") server_nonce = advance(item, 2);
        else if begins_with(item, "s=") salt_text = advance(item, 2);
        else if begins_with(item, "i=") iteration_text = advance(item, 2);
        else if begins_with(item, "m=") return auth_error(c, "the server asked for a SCRAM extension this client does not know");
    }

    if !begins_with(server_nonce, s.nonce) || server_nonce.count <= s.nonce.count {
        return auth_error(c, "the server's SCRAM nonce does not extend the client's");
    }

    salt, decoded := Base64.base64_decode(salt_text);

    defer free(salt);
    if !decoded || !salt return protocol_error(c, "a bad SCRAM salt");

    iterations, valid, extra := string_to_int(iteration_text);
    if !valid || extra || iterations < 1 return protocol_error(c, "a bad SCRAM iteration count");

    if iterations > config.max_scram_iterations {
        return auth_error(c, tprint("the server asked for % SCRAM iterations, more than max_scram_iterations (%)", iterations, config.max_scram_iterations));
    }

    salted := Hmac.pbkdf2_hmac_sha256(prepare_password(config.password), salt, iterations);
    salted_text := string.{ 32, salted.data };
    client_key := Hmac.hmac_sha256(salted_text, "Client Key");
    stored_key := Sha.sha256(string.{ 32, client_key.data });

    binding := Base64.base64_encode(s.binding,, temp);
    without_proof := tprint("c=%,r=%", binding, server_nonce);
    auth_message := tprint("%,%,%", s.client_first_bare, server_first, without_proof);
    client_signature := Hmac.hmac_sha256(string.{ 32, stored_key.data }, auth_message);
    proof: [32] u8;

    for 0..31 proof[it] = client_key[it] ^ client_signature[it];

    server_key := Hmac.hmac_sha256(salted_text, "Server Key");

    s.server_signature = Hmac.hmac_sha256(string.{ 32, server_key.data }, auth_message);

    final := tprint("%,p=%", without_proof, Base64.base64_encode(string.{ 32, proof.data },, temp));

    begin_message(c, #char "p");
    put_bytes(*c.out, final);
    end_message(c);

    return send(c);
}

scram_finish :: (c: *Connection, s: *Scram, data: [] u8) -> Pg_Error {
    if !s.started return protocol_error(c, "a SASL final message without a SASL start");

    text := cast(string) data;
    if begins_with(text, "e=") return auth_error(c, tprint("the server rejected SCRAM: %", advance(text, 2)));
    if !begins_with(text, "v=") return protocol_error(c, "a bad SCRAM final message");

    end := find_index_from_left(text, #char ",");
    encoded := ifx end < 0 then advance(text, 2) else slice(text, 2, end - 2);
    signature, decoded := Base64.base64_decode(encoded);

    defer free(signature);
    if !decoded || signature.count != 32 return auth_error(c, "the server's SCRAM signature is malformed");

    difference: u8;
    for 0..31 difference |= signature[it] ^ s.server_signature[it];
    if difference return auth_error(c, "the server's SCRAM signature is wrong: it does not know the password");

    s.verified = true;

    return .{};
}

// SASLprep (RFC 4013) would normalize non-ASCII passwords. Like libpq when normalization is not
// possible, the password's bytes are used as they are; ASCII passwords need nothing.
prepare_password :: (password: string) -> string {
    return password;
}

#if TLS {
    // The hash of the server's certificate for tls-server-end-point: the certificate's own
    // signature hash, with MD5 and SHA-1 raised to SHA-256. Empty when the signature uses no hash
    // (Ed25519) or one this client cannot compute.
    tls_server_end_point :: (stream: *Tls.Tls_Stream) -> string {
        text := cast(string) Tls.peer_certificate_der(stream);
        if !text return "";

        hash := Tls.peer_certificate_signature_hash(stream);
        if hash == "md5" || hash == "sha1" || hash == "sha256" {
            digest := Sha.sha256(text);
            return copy_string(string.{ digest.count, digest.data },, temp);
        } else if hash == "sha384" {
            digest := Sha.sha384(text);
            return copy_string(string.{ digest.count, digest.data },, temp);
        } else if hash == "sha512" {
            digest := Sha.sha512(text);
            return copy_string(string.{ digest.count, digest.data },, temp);
        }

        return "";
    }
}
