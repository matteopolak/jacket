"""Pinned TZif fixtures and zoneinfo vectors; run through build_portable.py data."""
from __future__ import annotations

import io
import struct
import zipfile
from datetime import datetime, timedelta, timezone
from zoneinfo import ZoneInfo

from repo_output import output_dir
from toolkit.download import pinned

URL = "https://files.pythonhosted.org/packages/5c/23/c7abc0ca0a1526a0774eca151daeb8de62ec457e77262b66b359c3c7679e/tzdata-2025.2-py2.py3-none-any.whl"
SHA256 = "1a403fada01ff9221ca8044d701868fa132215d84beb92242d9acd2147f667a8"
EPOCH = datetime(1970, 1, 1, tzinfo=timezone.utc)


def transitions(data):
    """Read transition instants, using the wide section where present."""
    header = struct.unpack_from(">6I", data, 20)
    width, start = 4, 44
    if data[4] != 0:
        utc, standard, leaps, count, types, chars = header
        start += count * 5 + types * 6 + chars + leaps * 8 + standard + utc
        header = struct.unpack_from(">6I", data, start + 20)
        start += 44
        width = 8
    count = header[3]
    return list(struct.unpack_from(f">{count}{'q' if width == 8 else 'i'}", data, start))


def fetch():
    """Write deterministic vectors and the wheel's zoneinfo under the output directory."""
    root = output_dir("time-conformance")
    stamp = root / ".complete"
    if stamp.exists() and stamp.read_text() == SHA256 + " vectors-v1\n":
        return root
    archive = pinned(URL, SHA256, "time/tzdata-2025.2.whl")
    root.mkdir(parents=True, exist_ok=True)
    records = []
    with zipfile.ZipFile(archive) as wheel:
        for entry in sorted(wheel.namelist()):
            prefix = "tzdata/zoneinfo/"
            if not entry.startswith(prefix):
                continue
            name = entry[len(prefix):]
            data = wheel.read(entry)
            if not data.startswith(b"TZif"):
                continue
            path = root / "zoneinfo" / name
            path.parent.mkdir(parents=True, exist_ok=True)
            path.write_bytes(data)
            zone = ZoneInfo.from_file(io.BytesIO(data), key=name)
            changes = set(transitions(data))
            # Find footer-generated changes too; daily sampling brackets transitions,
            # binary search locates their exact second. Stored changes are already exact.
            day = datetime(1900, 1, 1, tzinfo=timezone.utc)
            stop = datetime(2101, 1, 1, tzinfo=timezone.utc)
            old = day.astimezone(zone).utcoffset()
            while day < stop:
                following = day + timedelta(days=1)
                new = following.astimezone(zone).utcoffset()
                if new != old:
                    lo, hi = int((day - EPOCH).total_seconds()), int((following - EPOCH).total_seconds())
                    while hi - lo > 1:
                        mid = (lo + hi) // 2
                        if (EPOCH + timedelta(seconds=mid)).astimezone(zone).utcoffset() == old:
                            lo = mid
                        else:
                            hi = mid
                    changes.add(hi)
                old, day = new, following
            instants = {t + delta for t in changes for delta in (-1, 0, 1)}
            instants.update(int((datetime(y, m, 15, tzinfo=timezone.utc) - EPOCH).total_seconds())
                            for y in range(1900, 2101) for m in (1, 4, 7, 10))
            offsets, resolutions = [], []
            for second in sorted(instants):
                local = (EPOCH + timedelta(seconds=second)).astimezone(zone)
                abbreviation = local.tzname().encode()
                offsets.append(struct.pack(">qiB", second, int(local.utcoffset().total_seconds()), len(abbreviation)) + abbreviation)
            for second in sorted(changes):
                before = int((EPOCH + timedelta(seconds=second-1)).astimezone(zone).utcoffset().total_seconds())
                after = int((EPOCH + timedelta(seconds=second)).astimezone(zone).utcoffset().total_seconds())
                if before == after:
                    continue
                wall_second = second + min(before, after) + abs(after-before) // 2
                wall = (EPOCH + timedelta(seconds=wall_second)).replace(tzinfo=None)
                candidates = sorted(int((wall.replace(tzinfo=zone, fold=f).astimezone(timezone.utc)-EPOCH).total_seconds()) for f in (0, 1))
                gap = after > before
                resolutions.append(struct.pack(">qqqB", wall_second, candidates[0], candidates[1], gap))
            encoded = name.encode()
            records.append(struct.pack(">H", len(encoded)) + encoded + struct.pack(">I", len(offsets)) + b"".join(offsets)
                           + struct.pack(">I", len(resolutions)) + b"".join(resolutions))
    (root / "vectors.bin").write_bytes(struct.pack(">I", len(records)) + b"".join(records))
    stamp.write_text(SHA256 + " vectors-v1\n")
    return root


if __name__ == "__main__":
    print(fetch())
