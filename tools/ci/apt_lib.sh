# shellcheck shell=bash
# Shared by the CI install scripts (source it): apt-get that survives the hosted runners' stalling mirror.
#
# The runners' Ubuntu mirror sometimes stalls mid-transfer, and apt waits on a stalled connection
# indefinitely. Time out stalled requests and retry them; the setting stays for the job's later
# apt-get calls.
# Hosted images also configure optional vendor feeds. Keep the job's dependency
# repositories explicit rather than making unrelated feeds prerequisites of every update.
apt_configure() {
  local apt_root=${1:?apt_configure requires an APT configuration root} base name parts
  parts="$apt_root/jai-ci-sources.list.d"
  if [[ -s "$apt_root/sources.list.d/ubuntu.sources" ]]; then
    base="$apt_root/sources.list.d/ubuntu.sources"
    name=ubuntu.sources
  elif [[ -s "$apt_root/sources.list" ]]; then
    base="$apt_root/sources.list"
    name=ubuntu.list
  else
    echo 'Cannot configure CI APT: no Ubuntu base package sources found.' >&2
    return 1
  fi
  sudo install -d "$parts" "$apt_root/apt.conf.d"
  sudo rm -f "$parts/ubuntu.sources" "$parts/ubuntu.list"
  sudo install -m 0644 "$base" "$parts/$name"
  printf '%s\n' \
    'Acquire::Retries "5";' 'Acquire::http::Timeout "30";' 'Acquire::https::Timeout "30";' \
    'APT::Update::Error-Mode "any";' \
    'Dir::Etc::sourcelist "/dev/null";' \
    "Dir::Etc::sourceparts \"$parts\";" \
    | sudo tee "$apt_root/apt.conf.d/80jai-ci-network" > /dev/null
}

# Run an apt-get command, giving up on an attempt after three minutes and trying up to five times.
# An install cut off mid-way leaves dpkg half done, so finish that before the retry. Its output is
# shown only when an attempt fails.
apt_get() {
  local attempt log
  log=$(mktemp)
  for attempt in 1 2 3 4 5; do
    # shellcheck disable=SC2024 # the log is ours; only apt-get needs root
    if sudo timeout 180 apt-get "$@" > "$log" 2>&1; then
      rm -f "$log"
      return 0
    fi
    tail -n 40 "$log" >&2
    echo "apt-get $1 failed or stalled (attempt $attempt of 5)" >&2
    sudo dpkg --configure -a || true
  done
  return 1
}

# Best-effort: fetch the packages an install would need over several connections into apt's archive
# cache, so one slow connection cannot stall the whole install. The real install then finds the
# files (apt checks their hashes and refetches anything bad). Never fails; apt_get does the work.
apt_prefetch() {
  local uris
  uris=$(sudo apt-get install --yes --no-install-recommends --print-uris "$@" 2> /dev/null \
    | sed -n "s/^'\([^']*\)' \([^ ]*\) .*/\1 \2/p") || return 0
  [[ -n "$uris" ]] || return 0
  # shellcheck disable=SC2016 # $0 and $1 belong to the inner bash
  printf '%s\n' "$uris" \
    | xargs -P 8 -L 1 bash -c 'sudo timeout 180 curl --fail --silent --location --retry 3 --connect-timeout 20 --output "/var/cache/apt/archives/$1" "$0"' \
    || true
}
