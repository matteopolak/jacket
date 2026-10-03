# md5

## Public declaration inventory

Visible public declarations and module-conditional variants are listed below. Procedure bodies and aggregate initializers are omitted; complete public type members are listed separately.

- `md5 :: (input: string) -> string`

## Public type members

No public aggregate fields or enum members were identified.

## Behavioral contract and constraints

The module exposes MD5 digest operations over byte data. It is suitable only for compatibility/checksum uses, not collision-resistant security. Input is processed as bytes. The returned `string` is 32 hexadecimal ASCII characters representing the 16-byte MD5 digest; it is not the raw 16-byte binary digest. The module does not provide a cryptographic security guarantee.
