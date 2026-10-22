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

# Run an apt-get command, giving up on an attempt after ten minutes and retrying it twice. An
# install cut off mid-way leaves dpkg half done, so finish that before the retry.
apt_get() {
  local attempt
  for attempt in 1 2 3; do
    if sudo timeout 600 apt-get "$@"; then
      return 0
    fi
    echo "apt-get $1 failed or stalled (attempt $attempt of 3)" >&2
    sudo dpkg --configure -a || true
  done
  return 1
}
