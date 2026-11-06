# CommonMark executable examples

`spec.json` is the complete, unmodified CommonMark 0.31.2 executable-example set by John MacFarlane, from <https://spec.commonmark.org/0.31.2/spec.json>. It is licensed under Creative Commons Attribution-ShareAlike 4.0 International; see LICENSE.html and <https://spec.commonmark.org/0.31.2/>. No modifications were made to the examples. `manifest.json` pins bytes, IDs, source and the effective production parser profile. The license HTML is the full legal code from Creative Commons' own repository, pinned separately. One trailing HTML space is removed for repository whitespace checks; the manifest records both original and retained hashes.

The locked pulldown-cmark crate includes the same 652 Markdown inputs. Its generated expected HTML uses literal quotes instead of equivalent entities in 21 text nodes. The independent DOM comparator accepts equivalent serialization, never altered structure or text.

The fixture driver imports the actual producer document parser and production Astro Document component. The reference parser is a test-only dependency. Whole-site HTML validation, guide resolution, URL/base policy and browser parity are separate gates.

`extensions.json` preserves all 55 published provider table/strikethrough cases: GFM table 9, GFM strikethrough 3, provider table 28 and provider strikethrough 15. Their original source file hashes and crate checksum are recorded in the manifest; LICENSE-provider.txt retains the provider MIT notice. The GFM specification is based on John MacFarlane's CommonMark specification: <https://github.github.com/gfm/>. The shared specification license is also retained.

`transformations.json` pins the reviewed policy rules and every affected original case ID, derived from the independent reference AST and published extension HTML before comparing production output. The driver never updates these expectations. Unknown transform membership fails and requires source review.
