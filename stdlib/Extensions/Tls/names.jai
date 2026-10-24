// The names inside an X.509 certificate that a client matches a host name against: the DNS and IP
// address entries of the subject alternative name extension, and the subject's common name. For
// callers that apply a naming rule of their own (the library's own check is `verify_name`).

// Slices into the DER the names were read from: they are valid as long as it is.
Certificate_Names :: struct {
    // The dNSName entries, in the order the certificate lists them.
    dns: [..] string;
    // The iPAddress entries: 4 raw bytes for IPv4, 16 for IPv6.
    addresses: [..] string;
    // The first common name of the subject (UTF8String, PrintableString, IA5String and
    // TeletexString values; a BMPString only if every character is ASCII).
    common_name: string;
    has_common_name: bool;
    // Set when the common name could not be read as text (a zero byte inside it, an unknown
    // string type).
    common_name_unreadable: bool;
}

// Reads the names of the certificate in `der`. False when the DER is not a certificate.
certificate_names :: (der: [] u8, names: *Certificate_Names) -> bool {
    names.* = .{};

    certificate, ok := der_any(der);

    if !ok || certificate.tag != 0x30 return false;

    tbs: [] u8;
    tbs, ok = der_expect(certificate.content, 0x30);

    if !ok return false;

    fields := tbs;

    // version [0] OPTIONAL, serialNumber, signature, issuer, validity, subject, subjectPublicKeyInfo.
    element: Der_Element;
    element, ok = der_any(fields);

    if !ok return false;

    if element.tag == 0xa0 {
        fields = element.rest;
    }

    subject: [] u8;

    for 0..5 {
        element, ok = der_any(fields);

        if !ok return false;

        if it == 4 subject = element.content;

        fields = element.rest;
    }

    read_common_name(subject, names);

    // issuerUniqueID [1], subjectUniqueID [2], extensions [3].
    while fields.count > 0 {
        element, ok = der_any(fields);

        if !ok return false;

        fields = element.rest;

        if element.tag != 0xa3 continue;

        list: Der_Element;
        list, ok = der_any(element.content);

        if !ok || list.tag != 0x30 return false;

        read_extensions(list.content, names);
    }

    return true;
}

free_certificate_names :: (names: *Certificate_Names) {
    array_free(names.dns);
    array_free(names.addresses);

    names.* = .{};
}

#scope_module

Der_Element :: struct {
    tag: u8;
    content: [] u8;
    rest: [] u8; // what follows the element
}

// The element at the start of `data`, whatever its tag (a single byte tag, lengths up to 4 bytes).
der_any :: (data: [] u8) -> Der_Element, bool {
    if data.count < 2 return .{}, false;

    element: Der_Element;

    element.tag = data[0];

    header := 2;
    length := cast(s64) data[1];

    if length >= 0x80 {
        bytes := length & 0x7f;

        if bytes == 0 || bytes > 4 || data.count < 2 + bytes return .{}, false;

        length = 0;

        for 0..bytes - 1 length = length * 256 + cast(s64) data[2 + it];

        header = 2 + bytes;
    }

    if length > data.count - header return .{}, false;

    element.content = .{ length, data.data + header };
    element.rest = .{ data.count - header - length, data.data + header + length };

    return element, true;
}

// The content of the element at the start of `data` if its tag is `tag`.
der_expect :: (data: [] u8, tag: u8) -> [] u8, bool {
    element, ok := der_any(data);

    if !ok || element.tag != tag return .[], false;

    return element.content, true;
}

read_common_name :: (subject: [] u8, names: *Certificate_Names) {
    // Name ::= SEQUENCE OF RelativeDistinguishedName (SET OF AttributeTypeAndValue).
    rest := subject;

    while rest.count > 0 {
        set, ok := der_any(rest);

        if !ok || set.tag != 0x31 return;

        rest = set.rest;

        attributes := set.content;

        while attributes.count > 0 {
            attribute: Der_Element;
            attribute, ok = der_any(attributes);

            if !ok || attribute.tag != 0x30 return;

            attributes = attribute.rest;

            identifier: Der_Element;
            identifier, ok = der_any(attribute.content);

            if !ok || identifier.tag != 0x06 continue;
            if !oid_matches(identifier.content, .[0x55, 0x04, 0x03]) continue;

            value: Der_Element;
            value, ok = der_any(identifier.rest);

            if !ok return;

            names.has_common_name = true;

            text, readable := string_value(value);

            names.common_name = text;
            names.common_name_unreadable = !readable;

            return;
        }
    }
}

// The text of a directory string, as UTF-8 (a BMPString is read only when it is ASCII: its bytes
// are then in the DER's even positions, so it cannot be a slice; those are not supported).
string_value :: (value: Der_Element) -> string, bool {
    if value.tag == 0x0c || value.tag == 0x13 || value.tag == 0x16 || value.tag == 0x14 {
        text := string.{ value.content.count, value.content.data };

        for text if it == 0 return "", false;

        return text, true;
    }

    return "", false;
}

read_extensions :: (extensions: [] u8, names: *Certificate_Names) {
    rest := extensions;

    while rest.count > 0 {
        extension, ok := der_any(rest);

        if !ok || extension.tag != 0x30 return;

        rest = extension.rest;

        identifier: Der_Element;
        identifier, ok = der_any(extension.content);

        if !ok || identifier.tag != 0x06 continue;
        if !oid_matches(identifier.content, .[0x55, 0x1d, 0x11]) continue;

        // critical BOOLEAN DEFAULT FALSE, then the value in an OCTET STRING.
        next: Der_Element;
        next, ok = der_any(identifier.rest);

        if !ok continue;

        if next.tag == 0x01 {
            next, ok = der_any(next.rest);

            if !ok continue;
        }

        if next.tag != 0x04 continue;

        list: Der_Element;
        list, ok = der_any(next.content);

        if !ok || list.tag != 0x30 continue;

        entries := list.content;

        while entries.count > 0 {
            entry: Der_Element;
            entry, ok = der_any(entries);

            if !ok break;

            entries = entry.rest;

            text := string.{ entry.content.count, entry.content.data };

            if entry.tag == 0x82 {
                array_add(*names.dns, text);
            } else if entry.tag == 0x87 && (text.count == 4 || text.count == 16) {
                array_add(*names.addresses, text);
            }
        }
    }
}
