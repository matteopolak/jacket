#!/usr/bin/env python3
"""Generate private reservation hooks from the pinned official C SDK only."""

import argparse
import difflib
from pathlib import Path
from prepare_rpmalloc import ROOT, PATCHES, prepare


def generate(vendor, work):
    boundary = (
        "sdk-metadata.patch"
        if any(p[0] == "sdk-metadata.patch" for p in PATCHES)
        else None
    )
    prepared = prepare(vendor, work / "base", stop_before=boundary)
    source = (prepared / "rpmalloc.c").read_text()
    if source.count("_rpmalloc_mmap(") != 5:
        raise ValueError(
            "Review every pinned SDK mapping path before reserving metadata"
        )
    body = source

    def replace(old, new):
        nonlocal body
        if body.count(old) != 1:
            raise ValueError("Pinned SDK metadata hook changed: " + repr(old))
        body = body.replace(old, new)

    for size, spans in (
        ("aligned_span_count * _memory_span_size", "aligned_span_count"),
        ("block_size", "span_count"),
        ("num_pages * _memory_page_size", "1"),
        ("mapped_size", "1"),
    ):
        old = "_rpmalloc_mmap(" + size + ", &align_offset)"
        replace(
            old,
            "\n#if BINDING_RPMALLOC_RUNTIME_CONFIGURATION\n"
            "binding_metadata_map("
            + size
            + ", &align_offset, "
            + spans
            + ")\n#else\n"
            + old
            + "\n#endif\n",
        )
    replace(
        "span_t* span = 0;\n\t// If there are global reserved spans",
        "span_t* span = 0;\n#if BINDING_RPMALLOC_RUNTIME_CONFIGURATION\n"
        "\tBindingMetadataReservation heap_metadata = {0};\n#endif\n\t// If there are global reserved spans",
    )
    reserve = (
        "#if BINDING_RPMALLOC_RUNTIME_CONFIGURATION\n"
        "\t\tsize_t metadata_heaps = (_memory_span_size - sizeof(span_t)) / aligned_heap_size;\n"
        "\t\tif (metadata_heaps < request_heap_count) metadata_heaps = request_heap_count;\n"
        "\t\tif (!binding_metadata_reserve(&heap_metadata, metadata_heaps, 0)) return 0;\n#endif\n"
    )
    replace(
        "if (_memory_global_reserve_count >= heap_span_count) {\n",
        "if (_memory_global_reserve_count >= heap_span_count) {\n" + reserve,
    )
    # The heap request/page-size branch determines the exact native heap count
    # before any mapping; reserve once there if no global reserve was used.
    replace(
        "\t\tsize_t align_offset = 0;\n\t\tspan = (span_t*)",
        reserve + "\t\tsize_t align_offset = 0;\n\t\tspan = (span_t*)",
    )
    replace(
        "\n;\n\t\tif (!span)\n\t\t\treturn 0;",
        "\n;\n\t\tif (!span) {\n#if BINDING_RPMALLOC_RUNTIME_CONFIGURATION\n"
        "\t\t\tbinding_metadata_cancel(&heap_metadata);\n#endif\n\t\t\treturn 0;\n\t\t}",
    )
    replace(
        "\tif (span_count > heap_span_count) {\n",
        "#if BINDING_RPMALLOC_RUNTIME_CONFIGURATION\n"
        "\tsize_t metadata_heap_count = remain_size / aligned_heap_size;\n"
        "\tif (metadata_heap_count < request_heap_count) metadata_heap_count = request_heap_count;\n"
        "\tbinding_metadata_commit_heaps(&heap_metadata, heap, aligned_heap_size, metadata_heap_count);\n"
        "#endif\n\tif (span_count > heap_span_count) {\n",
    )
    replace(
        "_rpmalloc_unmap(void* address, size_t size, size_t offset, size_t release) {\n",
        "_rpmalloc_unmap(void* address, size_t size, size_t offset, size_t release) {\n"
        "#if BINDING_RPMALLOC_RUNTIME_CONFIGURATION\n\tbinding_metadata_unmap(address, size, release);\n#endif\n",
    )
    replace(
        "rpmalloc_thread_initialize(void) {\n",
        "rpmalloc_thread_initialize(void) {\n#if BINDING_RPMALLOC_RUNTIME_CONFIGURATION\n"
        "\tif (!binding_domain_prepare_heap()) return;\n#endif\n",
    )
    return "".join(
        difflib.unified_diff(
            source.splitlines(True),
            body.splitlines(True),
            fromfile="a/rpmalloc.c",
            tofile="b/rpmalloc.c",
        )
    )


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--vendor", type=Path, required=True)
    parser.add_argument("--work", type=Path, required=True)
    parser.add_argument("--check", action="store_true")
    args = parser.parse_args()
    path = ROOT / "stdlib/internal/bindings/rpmalloc-heaps/sdk-metadata.patch"
    text = generate(args.vendor, args.work)
    if args.check:
        if path.read_text() != text:
            raise SystemExit("Stale metadata reservation patch")
    else:
        path.write_text(text)


if __name__ == "__main__":
    main()
