#!/bin/sh
# Regenerates the test certificates and keys in this directory with the openssl CLI:
#
#     sh make_certs.sh
#
# Everything here is for the tests only. The private keys are public (they are committed), so no
# certificate from this directory may be trusted for anything but the tests of Extensions/Tls and
# Extensions/Http. They are valid from 2020-01-01 (so a machine with a wrong clock still accepts
# them) to 2120-01-01, except expired.pem, which ended in 2001.
set -eu
cd "$(dirname "$0")"

START=20200101000000Z
END=21200101000000Z
work=$(mktemp -d)
trap 'rm -rf "$work"' EXIT

names="ca.pem ca.key other_ca.pem other_ca.key server.pem server.key rsa.pem rsa.key wrong_name.pem wrong_name.key expired.pem expired.key unknown.pem unknown.key client.pem client.key"
rm -f $names

mkdir "$work/db"
: > "$work/db/index.txt"
echo 01 > "$work/db/serial"

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
        openssl ca -batch -config "$work/ca.cnf" -cert $signer.pem -keyfile $signer.key -in "$csr" \
            -startdate $start -enddate $end -extensions ext -notext -out "$out" 2>/dev/null
    else
        openssl ca -batch -config "$work/ca.cnf" -selfsign -keyfile "$ISSUER_KEY" -in "$csr" \
            -startdate $start -enddate $end -extensions ext -notext -out "$out" 2>/dev/null
    fi
}

# A certificate authority: ca.pem signs everything below; other_ca.pem is one nobody trusts.
for ca in ca other_ca; do
    ec $ca.key
    openssl req -new -key $ca.key -subj "/CN=Jai Test $ca" -out "$work/$ca.csr"
    ISSUER_KEY=$ca.key sign "$work/$ca.csr" "basicConstraints=critical,CA:TRUE
keyUsage=critical,keyCertSign,cRLSign" $ca.pem $START $END
done

# leaf NAME CA SAN EKU [KEYTYPE [START END]]: a leaf certificate NAME.pem signed by CA.
leaf() {
    name=$1; ca=$2; san=$3; eku=$4; kind=${5:-ec}; start=${6:-$START}; end=${7:-$END}
    if [ "$kind" = rsa ]; then openssl genrsa -out $name.key 2048 2>/dev/null; else ec $name.key; fi
    openssl req -new -key $name.key -subj "/CN=$name.test" -out "$work/$name.csr"
    sign "$work/$name.csr" "basicConstraints=CA:FALSE
keyUsage=critical,digitalSignature,keyEncipherment
extendedKeyUsage=$eku
subjectAltName=$san" $name.pem $start $end $ca
}

SERVER_NAMES="DNS:localhost,DNS:server.test,IP:127.0.0.1,IP:::1"
leaf server ca "$SERVER_NAMES" serverAuth
leaf rsa ca "$SERVER_NAMES" serverAuth rsa
leaf wrong_name ca "DNS:other.test" serverAuth
leaf unknown other_ca "$SERVER_NAMES" serverAuth
leaf client ca "DNS:client.test" clientAuth
leaf expired ca "$SERVER_NAMES" serverAuth ec 20000101000000Z 20010101000000Z

echo "wrote: $(ls $names | tr '\n' ' ')"
