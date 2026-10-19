# shellcheck shell=bash
# Shared by the CI install scripts (source it): apt-get that survives the hosted runners' stalling mirror.
#
# The runners' Ubuntu mirror sometimes stalls mid-transfer, and apt waits on a stalled connection
# indefinitely. Time out stalled requests and retry them; the setting stays for the job's later
# apt-get calls.
printf '%s\n' 'Acquire::Retries "5";' 'Acquire::http::Timeout "30";' 'Acquire::https::Timeout "30";' \
  | sudo tee /etc/apt/apt.conf.d/80jai-ci-network > /dev/null

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
