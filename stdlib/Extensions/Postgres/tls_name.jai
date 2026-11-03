// Whether the server's certificate is for the host the client connected to (sslmode=verify-full),
// by the rules of libpq rather than the TLS library's:
//
//   - a host name is compared with the certificate's dNSName entries; an IP address with its
//     iPAddress entries (by value, so `192.000.002.001` is `192.0.2.1`) and, as plain text, with
//     its dNSName entries;
//   - the subject's common name is looked at only when the certificate has no entry of the kind
//     that could match this host: no dNSName for a host name, no iPAddress for an address;
//   - `*` stands for one whole label at the start of a name (`*.example.com`), never for an address.
//
// The messages are libpq's.

#if TLS {
    // "" when the certificate is for `host`, else why not.
    //
    // - stream: the finished TLS handshake, whose peer certificate is checked.
    // - host: the host name or address the client connected to.
    verify_server_name :: (stream: *Tls.Tls_Stream, host: string) -> string {
        der := Tls.peer_certificate_der(stream);
        names: Tls.Certificate_Names;

        if !Tls.certificate_names(der, *names) return "could not read the server's certificate";

        defer Tls.free_certificate_names(*names);

        wanted := host;

        // A name may end in the root label.
        if wanted.count > 1 && wanted[wanted.count - 1] == #char "." wanted.count -= 1;

        address, is_address := host_address(wanted);

        examined := 0;
        first := "";
        found := false;

        for names.dns {
            examined += 1;

            if first.count == 0 first = it;
            if name_matches(it, wanted, is_address) found = true;
        }

        for names.addresses {
            examined += 1;

            if first.count == 0 first = address_text(it);
            if is_address && address_matches(it, address) found = true;
        }

        // The common name counts only where the certificate has no entry for this kind of host.
        has_entry := ifx is_address then names.addresses.count > 0 else names.dns.count > 0;

        if !found && !has_entry && names.has_common_name {
            if names.common_name_unreadable return "SSL certificate's common name contains embedded null";

            examined += 1;

            if first.count == 0 first = names.common_name;
            if name_matches(names.common_name, wanted, is_address) found = true;
        }

        if found return "";

        if examined == 0 return "could not get server's host name from server certificate";

        if examined == 1 {
            return tprint("server certificate for \"%\" does not match host name \"%\"", first, host);
        }

        return tprint("server certificate for \"%\" (and % other name%) does not match host name \"%\"", first, examined - 1, ifx examined == 2 then "" else "s", host);
    }

    // The address `host` stands for and true, or false when `host` is a name.
    //
    // - host: the host name or address, without a trailing dot.
    host_address :: (host: string) -> Io.Address, bool {
        colons := 0;

        for host {
            if it == #char ":" colons += 1;
            else if it != #char "." && (it < #char "0" || it > #char "9") && !is_hex_letter(it) return .{}, false;
        }

        // A single colon would read as a port.
        if colons == 1 return .{}, false;

        address, ok := Io.parse_address(host);

        if !ok return .{}, false;

        // `parse_address` reads a bare number such as "1" or a port-only text as an address too.
        if address.family == .IPV4 {
            dots := 0;

            for host if it == #char "." dots += 1;

            if dots != 3 return .{}, false;
        }

        return address, true;
    }

    // Whether `c` is `a` to `f` in either case.
    //
    // - c: the byte to test.
    is_hex_letter :: (c: u8) -> bool {
        return (c >= #char "a" && c <= #char "f") || (c >= #char "A" && c <= #char "F");
    }

    // Whether the certificate entry is `address`, byte for byte.
    //
    // - raw: an iPAddress entry of the certificate: 4 or 16 bytes.
    // - address: the address the client connected to.
    address_matches :: (raw: string, address: Io.Address) -> bool {
        length := ifx address.family == .IPV4 then 4 else 16;

        if raw.count != length return false;

        for 0..length - 1 {
            if raw[it] != address.bytes[it] return false;
        }

        return true;
    }

    // The text of a raw address, as libpq shows it in its messages.
    //
    // - raw: an iPAddress entry of the certificate: 4 or 16 bytes.
    address_text :: (raw: string) -> string {
        address: Io.Address;
        address.family = ifx raw.count == 4 then .IPV4 else .IPV6;

        for 0..raw.count - 1 address.bytes[it] = raw[it];

        text := Io.address_to_string(address);

        if address.family == .IPV6 {
            // "[2001:db8::1]:0"
            return string.{ text.count - 4, text.data + 1 };
        }

        // "192.0.2.1:0"
        return string.{ text.count - 2, text.data };
    }

    // A name from the certificate against the host: equal without regard to case, or a wildcard
    // `*.rest` for a host with exactly one more label in front of `.rest`.
    //
    // - certificate_name: a dNSName entry or common name from the certificate.
    // - host: the host name or address the client connected to.
    // - host_is_address: `host` is an address: a wildcard name never matches it.
    name_matches :: (certificate_name: string, host: string, host_is_address: bool) -> bool {
        name := certificate_name;

        if name.count > 1 && name[name.count - 1] == #char "." name.count -= 1;

        if equal_nocase(name, host) return true;

        if host_is_address || name.count < 3 || name[0] != #char "*" || name[1] != #char "." return false;

        // Nothing else may be a wildcard.
        for 1..name.count - 1 {
            if name[it] == #char "*" return false;
        }

        dot := -1;

        for host {
            if it == #char "." {
                dot = it_index;
                break;
            }
        }

        // The first label must exist.
        if dot < 1 return false;

        rest := string.{ host.count - dot, host.data + dot };
        suffix := string.{ name.count - 1, name.data + 1 };

        return equal_nocase(suffix, rest);
    }
}
