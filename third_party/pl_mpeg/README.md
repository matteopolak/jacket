# Official C algorithm input

Vendored official public phoboslab/pl_mpeg C source for permitted codec algorithm work, never a Jai implementation.

Source: https://raw.githubusercontent.com/phoboslab/pl_mpeg/master/pl_mpeg.h

License: upstream declares SPDX-License-Identifier: MIT and attributes Dominic
Szablewski (https://phoboslab.org). The unmodified header retains that attribution;
MIT.txt contains the standard SPDX MIT text (including its template copyright
line, not an invented upstream copyright year). The upstream repository provides
no separate LICENSE file. Standard text source:
https://raw.githubusercontent.com/spdx/license-list-data/main/text/MIT.txt

Source SHA-256: 3a8cb30c83c2a1147719c30fe0c8b93da2987aa43140077c575b39aaa75fc2c9

Fetched 2026-10-14. The vendored snapshot is unmodified; new Jai adaptations are separately identified and preserve the reviewed Jai interfaces rather than assuming C structure ABI equivalence.

Secondary official public C algorithm consultation (2026-10-14):
[FFmpeg MPEG audio decoder](https://raw.githubusercontent.com/FFmpeg/FFmpeg/master/libavcodec/mpegaudiodec_template.c),
copyright Fabrice Bellard, LGPL-2.1-or-later. Consulted to verify separate channel
scalefactors for shared Layer II sample codes and CRC coverage. No FFmpeg bodies,
tables or binaries are incorporated or executed. Jai compatibility signatures and
public constant data continue to come only from the reviewed specifications.
