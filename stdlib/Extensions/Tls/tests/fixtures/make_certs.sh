#!/bin/sh
# Regenerates the test certificates and keys in this directory with the openssl CLI:
#
#     sh make_certs.sh              # new keys and certificates, and certs.jai
#     sh make_certs.sh --jai-only   # only certs.jai, from the files that are here
#     sh make_certs.sh --extra-only # the files added after the first ones (inter, deep, revoked,
#                                   # the CRLs, client_encrypted.key), from the CAs that are here
#
# Everything here is for the tests only. The private keys are public (they are committed), so no
# certificate from this directory may be trusted for anything but the tests of Extensions/Tls and
# Extensions/Http. They are valid from 2020-01-01 (so a machine with a wrong clock still accepts
# them) to 2120-01-01, except expired.pem, which ended in 2001.
set -eu
cd "$(dirname "$0")"

# A word list on purpose: it is split into file names below.
base="ca.pem ca.key other_ca.pem other_ca.key server.pem server.key rsa.pem rsa.key wrong_name.pem wrong_name.key expired.pem expired.key unknown.pem unknown.key client.pem client.key"
extra="inter.pem inter.key deep.pem deep.key revoked.pem revoked.key ca.crl other_ca.crl client_encrypted.key"
names="$base $extra"

if [ "${1:-}" != --jai-only ]; then
if [ "${1:-}" = --extra-only ]; then
# shellcheck disable=SC2086
rm -f $extra certs.jai
else
# shellcheck disable=SC2086
rm -f $names certs.jai
fi
START=20200101000000Z
END=21200101000000Z
work=$(mktemp -d)
trap 'rm -rf "$work"' EXIT


mkdir "$work/db"
: > "$work/db/index.txt"
# The extra files are made on top of the base ones, from a later run: keep their serial numbers apart.
if [ "${1:-}" = --extra-only ]; then echo 0100 > "$work/db/serial"; else echo 01 > "$work/db/serial"; fi

ec() { openssl ecparam -name prime256v1 -genkey -noout -out "$1" 2>/dev/null; }

# sign CSR EXTENSIONS OUT START END [SIGNER]: signs a request with the certificate authority SIGNER
# (default: self-signed with the request's own key, `ISSUER_KEY`) using the `openssl ca` database,
# which unlike `openssl x509` can back-date a certificate.
sign() {
    csr=$1; ext=$2; out=$3; start=$4; end=$5; signer=${6:-}
    cat > "$work/ca.cnf" <<CONF
[ca]
default_ca = test
[test]
database = $work/db/index.txt
new_certs_dir = $work/db
serial = $work/db/serial
default_md = sha256
policy = anything
unique_subject = no
copy_extensions = none
[anything]
commonName = supplied
[ext]
$ext
CONF
    if [ -n "$signer" ]; then
        openssl ca -batch -config "$work/ca.cnf" -cert "$signer.pem" -keyfile "$signer.key" -in "$csr" \
            -startdate "$start" -enddate "$end" -extensions ext -notext -out "$out" 2>/dev/null
    else
        openssl ca -batch -config "$work/ca.cnf" -selfsign -keyfile "$ISSUER_KEY" -in "$csr" \
            -startdate "$start" -enddate "$end" -extensions ext -notext -out "$out" 2>/dev/null
    fi
}

if [ "${1:-}" != --extra-only ]; then
# A certificate authority: ca.pem signs everything below; other_ca.pem is one nobody trusts.
for ca in ca other_ca; do
    ec "$ca.key"
    openssl req -new -key "$ca.key" -subj "/CN=Jai Test $ca" -out "$work/$ca.csr"
    ISSUER_KEY=$ca.key sign "$work/$ca.csr" "basicConstraints=critical,CA:TRUE
keyUsage=critical,keyCertSign,cRLSign" "$ca.pem" "$START" "$END"
done

fi

# leaf NAME CA SAN EKU [KEYTYPE [START END]]: a leaf certificate NAME.pem signed by CA.
leaf() {
    name=$1; ca=$2; san=$3; eku=$4; kind=${5:-ec}; start=${6:-$START}; end=${7:-$END}
    if [ "$kind" = rsa ]; then openssl genrsa -out "$name.key" 2048 2>/dev/null; else ec "$name.key"; fi
    openssl req -new -key "$name.key" -subj "/CN=$name.test" -out "$work/$name.csr"
    sign "$work/$name.csr" "basicConstraints=CA:FALSE
keyUsage=critical,digitalSignature,keyEncipherment
extendedKeyUsage=$eku
subjectAltName=$san" "$name.pem" "$start" "$end" "$ca"
}

SERVER_NAMES="DNS:localhost,DNS:server.test,IP:127.0.0.1,IP:::1"
if [ "${1:-}" != --extra-only ]; then
leaf server ca "$SERVER_NAMES" serverAuth
leaf rsa ca "$SERVER_NAMES" serverAuth rsa
leaf wrong_name ca "DNS:other.test" serverAuth
leaf unknown other_ca "$SERVER_NAMES" serverAuth
leaf client ca "DNS:client.test" clientAuth
leaf expired ca "$SERVER_NAMES" serverAuth ec 20000101000000Z 20010101000000Z
fi

# An intermediate CA under ca.pem, with a leaf below it: trusting inter.pem alone is trusting a
# certificate that is not self-signed.
ec inter.key
openssl req -new -key inter.key -subj "/CN=Jai Test inter" -out "$work/inter.csr"
sign "$work/inter.csr" "basicConstraints=critical,CA:TRUE
keyUsage=critical,keyCertSign,cRLSign" inter.pem "$START" "$END" ca
leaf deep inter "$SERVER_NAMES" serverAuth

# A leaf of ca.pem that its CRL lists as revoked; other_ca's CRL lists nothing.
leaf revoked ca "$SERVER_NAMES" serverAuth

# crl CA OUT [REVOKED]: the revocation list of CA, listing the certificate REVOKED if given.
crl() {
    dir="$work/crl_$1"
    mkdir -p "$dir"
    : > "$dir/index.txt"
    echo 01 > "$dir/crlnumber"
    if [ -n "${3:-}" ]; then
        serial=$(openssl x509 -in "$3" -noout -serial | sed 's/^serial=//')
        printf 'R\t21200101000000Z\t200101000000Z\t%s\tunknown\t/CN=revoked.test\n' "$serial" > "$dir/index.txt"
    fi
    cat > "$dir/ca.cnf" <<CONF
[ca]
default_ca = test
[test]
database = $dir/index.txt
crlnumber = $dir/crlnumber
default_md = sha256
default_crl_days = 36500
CONF
    openssl ca -config "$dir/ca.cnf" -cert "$1.pem" -keyfile "$1.key" -gencrl \
        -crl_lastupdate "$START" -crl_nextupdate "$END" -out "$2" 2>/dev/null
}
crl ca ca.crl revoked.pem
crl other_ca other_ca.crl

# A key that is encrypted with the password "secret" (PKCS#8, PBES2 with PBKDF2 and AES-256).
openssl pkcs8 -topk8 -in client.key -out client_encrypted.key -v2 aes-256-cbc -v2prf hmacWithSHA256 -passout pass:secret

fi

# certs.jai: the same files as string constants, so the test programs carry them (no file is read
# when a program is built or run, which a cross build for another OS could not do). The files stay
# for openssl.
# shellcheck disable=SC2086
{
    echo "// Generated by make_certs.sh from the files in this directory: test-only certificates and keys."
    echo "Data_File :: struct {"
    echo "    name: string;"
    echo "    text: string;"
    echo "}"
    echo
    echo "DATA :: Data_File.["
    for f in $names; do echo "    .{ \"$f\", $(echo "$f" | tr 'a-z.' 'A-Z_') },"; done
    echo "];"
    for f in $names; do
        echo
        echo "$(echo "$f" | tr 'a-z.' 'A-Z_') :: #string END_OF_FILE"
        cat "$f"
        echo "END_OF_FILE"
    done
} > certs.jai

echo "wrote: $names certs.jai"
