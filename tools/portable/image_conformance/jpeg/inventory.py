"""The JPEG gate's inventory: pinned source files and the cases derived from them.

`manifest.json` pins each source set by repository, revision and the SHA-256 of every file. A case is
every `.jpg`/`.jpeg` among those files; its class comes from its folder and its expectation from its
class, unless the manifest lists an exception. Only facts that cannot be read from the files (reference
baselines, reviewed expectations, unsupported processes) are stored per case.

    manifest = json.loads(Path("manifest.json").read_text())
    for case in cases(manifest): ...
"""

import urllib.parse

CASE_SUFFIXES = (".jpg", ".jpeg")
CASE_FOLDERS = ("invalid", "valid", "crash-repro", "non-conformant")
EXPECTATION = {
    "invalid": "reject",
    "valid": "accept",
    "crash-repro": "observe",
    "non-conformant": "observe",
    "upstream-reference": "accept",
}
# What the reference decoder printed on a well-formed input; only deviations are pinned per case.
CLEAN_BASELINE = {"exit": 0, "warnings": 0, "stderr": ""}


def sources(manifest):
    """Every pinned file as {local path: {"url", "sha256"}}; the local path is `<set>/<repository path>`."""
    found = {}
    for source_set in manifest["sets"]:
        base = source_set["repository"].replace("https://github.com/", "")
        for path, digest in source_set["files"].items():
            url = f"https://raw.githubusercontent.com/{base}/{source_set['revision']}/{path}"
            found[f"{source_set['name']}/{path}"] = {
                "url": urllib.parse.quote(url, safe=":/%"),
                "sha256": digest,
            }
    return found


def case_class(local_path):
    """The case class of a pinned file, or None when it is not a JPEG case."""
    if not local_path.lower().endswith(CASE_SUFFIXES):
        return None
    parts = local_path.split("/")
    if parts[:2] == ["codec-corpus", "jpeg-conformance"] and parts[2] in CASE_FOLDERS:
        return parts[2]
    if parts[:2] == ["libjpeg-turbo", "testimages"]:
        return "upstream-reference"
    return None


def cases(manifest):
    """The gate's cases, in path order, each with its class, expectation and reference baseline."""
    pins = sources(manifest)
    exceptions = manifest["expectations"]
    unsupported = set(manifest["unsupported_process"])
    baselines = manifest["baseline_reference"]
    provenance = manifest["expectation_provenance"]
    found = []
    for local_path in sorted(pins):
        kind = case_class(local_path)
        if kind is None:
            continue
        case = {
            "id": local_path,
            "sha256": pins[local_path]["sha256"],
            "class": kind,
            "expectation": exceptions.get(local_path, EXPECTATION[kind]),
            "unsupported_process": local_path in unsupported,
            "baseline_reference": baselines.get(local_path, CLEAN_BASELINE),
        }
        if local_path in provenance:
            case["expectation_provenance"] = provenance[local_path]
        found.append(case)
    return found


def check_corpus(manifest, corpus, digest):
    """Errors for pinned files that are missing or changed, JPEGs the manifest does not pin, and a wrong case count.

    `digest(path)` is the SHA-256 of a file, for example `toolkit.hashing.sha256_file`.
    """
    errors = []
    for local_path, pin in sources(manifest).items():
        path = corpus / local_path
        if not path.is_file() or digest(path) != pin["sha256"]:
            errors.append("missing or changed: " + local_path)
    pinned = {case["id"] for case in cases(manifest)}
    present = {
        path.relative_to(corpus).as_posix() for path in corpus.rglob("*") if path.is_file()
    }
    errors += ["unpinned JPEG: " + name for name in sorted(present - pinned) if case_class(name)]
    if len(pinned) != manifest["case_count"]:
        errors.append(f"expected {manifest['case_count']} cases, the pins define {len(pinned)}")
    return errors
