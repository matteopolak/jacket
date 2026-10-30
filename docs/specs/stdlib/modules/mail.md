# Mail

## Public declaration inventory

Visible public declarations and module-conditional variants are listed below. Procedure bodies and aggregate initializers are omitted; complete public type members are listed separately.

- `Mail_Server :: struct`
- `Mail_Message :: struct`
- `mail_send :: (smtp: Mail_Server, msg: Mail_Message) -> bool`
- `scramble :: (text: string, secret: string) -> string`

## Public type members

#### `Mail_Server` (struct)

- `host:       string`
- `port:       int`
- `enable_ssl: bool`
- `ssl_ca_bundle_path: string`
- `username:   string`
- `password:   string`
- `verbose_logging := false`
- `timeout_milliseconds := 0`

#### `Mail_Message` (struct)

- `from:     string`
- `reply_to: string`
- `to:       [] string`
- `cc:       [] string`
- `bcc:      [] string`
- `subject:  string`
- `body:     string`
- `attachments: [] string`
- `html:     bool`

## Behavioral contract and constraints

The module sends plain-text or HTML email with optional CC/BCC and file attachments. Inputs describe message headers/body and attachment content; the module guarantees no transport security or delivery guarantees. Network/SMTP failures are exposed through the send result/status; recipient validation and ownership of attachment data follow the declared API and remain caller responsibilities.

`scramble(text, secret)` returns a same-length temporary string using a reversible XOR mapping over exactly 64 permitted characters: space, `!`, decimal digits, uppercase letters, and lowercase letters. Each input character is mapped to a value from 0 through 63; the corresponding secret character is selected cyclically, its mapped value is XORed with the input value, and the result is mapped back to that alphabet. Thus both strings must contain only those characters; unsupported characters assert. A non-empty `text` requires a non-empty `secret` because secret characters are selected cyclically. Scrambling the result again with the same secret recovers the original text. This is an obfuscation transform, not secure encryption.

### Operation details

- `mail_send` blocks until the SMTP exchange finishes, or until `timeout_milliseconds` (when above 0) has passed for the whole send, connecting included; 0 sets no limit. It returns false and logs one `.ERROR` line through `context.logger` for every failure. Headers (`from`, `reply_to`, `subject`, recipients) and the host reject NUL, CR and LF before anything is sent; an empty recipient list or a port outside 0 to 65535 is also rejected up front.
- `port` 0 means the protocol default. `enable_ssl` requires STARTTLS, and a connection that starts as TLS (`smtps`, port 465) is not offered: it would need a TLS-terminating test server to verify; without it credentials and message travel in clear text. Environment proxies and `.netrc` are ignored.
- `bcc` recipients are added to the envelope only. Attachments are read from disk at send time and sent as base64 parts; the body is quoted-printable UTF-8.
- All strings are borrowed for the call. `scramble` returns temporary storage, which the caller does not free and which is invalid after the temporary storage is reset.
