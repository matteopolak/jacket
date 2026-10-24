#!/bin/sh
# Starts a throwaway PostgreSQL cluster for tests/server.jai and prints the variables to export.
#
#     sh stdlib/Extensions/Postgres/tests/start_server.sh /tmp/pgtest 54329
#
# PG_BIN may point at the server's bin directory; otherwise initdb and pg_ctl come from PATH.
# Stop it with `pg_ctl -D <dir> stop`.
set -eu
dir=${1:?usage: start_server.sh <data directory> [port]}
port=${2:-54329}
bin=${PG_BIN:+$PG_BIN/}
here=$(cd "$(dirname "$0")" && pwd)
fixtures="$here/../../Tls/tests/fixtures"
export LC_ALL="${LC_ALL:-en_US.UTF-8}"

rm -rf "$dir"
"${bin}initdb" -D "$dir" -U postgres -A trust --no-sync > /dev/null
mkdir -p "$dir/socket"
cp "$fixtures/server.pem" "$dir/server.crt"
cp "$fixtures/server.key" "$dir/server.key"
chmod 600 "$dir/server.key"
cat > "$dir/pg_hba.conf" <<HBA
host      all  jai_md5        127.0.0.1/32  md5
host      all  jai_scram      127.0.0.1/32  scram-sha-256
host      all  jai_clear      127.0.0.1/32  password
hostssl   all  jai_tls_scram  127.0.0.1/32  scram-sha-256
hostnossl all  jai_tls_scram  127.0.0.1/32  reject
local     all  all                          trust
host      all  all            127.0.0.1/32  trust
HBA
cat >> "$dir/postgresql.conf" <<CONF
listen_addresses = '127.0.0.1'
port = $port
unix_socket_directories = '$dir/socket'
ssl = on
ssl_cert_file = 'server.crt'
ssl_key_file = 'server.key'
fsync = off
CONF
"${bin}pg_ctl" -D "$dir" -l "$dir/server.log" -w start > /dev/null
"${bin}psql" -h "$dir/socket" -p "$port" -U postgres -X -q -v ON_ERROR_STOP=1 <<SQL
set password_encryption = 'md5';
create role jai_md5 login password 'md5 secret';
create role jai_clear login password 'clear secret';
set password_encryption = 'scram-sha-256';
create role jai_scram login password 'scram secret';
create role jai_tls_scram login password 'tls secret';
SQL
echo "export JAI_PG_TEST_PORT=$port JAI_PG_TEST_SOCKET_DIR=$dir/socket"
