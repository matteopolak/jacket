# Documentation export

Documentation has two versioned contracts: `jai-doc-model/1` carries compiler facts to Jacket, and `jacket-doc-site/1` carries resolved documentation data to Astro. The compiler does not render HTML, and the site builder does not parse Jai or resolve declarations.

## Compiler facts

`jaic -doc_model DIR` emits a module object with `schema: "jai-doc-model"` and `schemaVersion: 1`. The authority is `crates/jaic-syntax/src/doc_comments/model.rs`; Jacket's strict wire DTOs live in `crates/jacket/src/doc/model/wire.rs`.

The model records declaration identities, kinds, overload parameters/results, comments, references, visibility, platform limits, and the files read during extraction. Each dependency has its compiler file identity and a lowercase SHA-256 digest of its contents. A same-length source edit still changes the digest. Jacket carries each module's dependency facts into the site export and records effective configuration and conservatively included source inputs in the manifest. These hashes describe inputs; they do not promise an incremental cache or permit a consumer to reuse stale pages.

References are four-column tuples: `[writtenName, declarationFile, declarationLine, private]`. They identify the declaration the compiler resolved, including aliases, rather than a URL guessed from a name. Invalid tuple lengths, types, declaration kinds, versions and dependency digests fail ingestion. Unknown fields are errors. Defaults apply only to fields the producer deliberately omits when empty. Struct, union, enum and enum-flags records require nonempty declaration token facts. The old raw source-layout `declaration` field is neither serialized nor accepted by the strict wire consumer; source locations and source links remain separate facts.

Each written type has a `definitions` list rather than dropping later platform variants. A definition records its actual kind, documentation, source bounds/digest, enclosing guards, generic scope, members and resolved references. Its token records contain exact spelling, source bounds, parser statement/name identities and reference-use flags. Each grouped member belongs to its actual name token; nested declarations cannot borrow outer member identities. Source bounds identify private formatter input, not public layout instructions. Guard bounds include the complete concrete expression, including parentheses around operands.

In `Collision: Collision`, the first token declares a field and the second uses a type. Only the use is eligible for the type's resolved link, even though both spellings match. Astro does not infer that distinction from punctuation or names.

Changing a field comment or indentation can leave the declaration's lexical spellings unchanged. The comment remains a complete documentation fact, while the eventual formatter independently chooses its line breaks.

## Site contract

`jacket doc --export-only --export /tmp/library-export` resolves the compiler model and configured Markdown guides without requiring Node by default. Explicit example validation can select a Node-based browser runtime. The export contains a manifest, compact site/route data, module and document shards, search facts, and copied assets. The Rust authority is `crates/jacket/src/doc/export/contract.rs`; the generated schema is `website/docs/schema/doc-site.schema.json`.

The manifest inventories every shard and asset by relative path, byte length and SHA-256. It records producer versions, revision/dirty state, effective configuration digest, source inputs, partial-build state and declared capabilities. Local compiler paths stay in the private extraction stage; public provenance uses portable paths. A partial module selection is explicitly marked and cannot stand in for a complete publication build.

Page targets refer to admitted route IDs and optional anchors; assets and external URLs are separate target variants. Route records keep the site graph compact. Full metadata belongs to the owning module, declaration or document shard. Search entries contain compiler-derived preview facts rather than HTML or a duplicate API model; navigation shards contain only facts needed for a bounded peer list.

A module's `id` identifies its route under the selected documentation library; its `importPath` is the executable import used in the import hint and examples. Selecting a standard-library module directly can give `id: "Json"` and `importPath: "Extensions/Json"`. External nested libraries retain their selected import path, such as `Acme/Widget`. Compiler documentation-link keys are a third, internal identity: the producer resolves them to the selected route and never substitutes them for external imports.

Markdown becomes tagged `DocumentNode` data: paragraphs, headings, lists, tables, links, code, examples and comparisons. Raw HTML is literal text. `ResolvedText` contains the original string and nonoverlapping UTF-8 byte spans for token roles and resolved targets. The consumer validates boundaries before slicing; character indices cannot substitute for byte offsets.

Structured publication URLs are optional semantic facts. Without a configured origin, canonical, article, website and breadcrumb URLs remain unknown; Astro omits their JSON-LD URL fields rather than inventing empty or relative destinations.

A declaration's `definitions` contain formatter-authored `code: ResolvedText`, semantic anchors, kind, documentation, scope, platforms and portable source identity. Jacket verifies compiler-pinned source bytes, supplies equivalent enclosing guards and formats all unique declarations with one native `jaifmt --declarations` batch. The documentation profile uses four-space indentation and an 88-column target with configuration discovery disabled; protected comment code/Markdown and literal contents remain intact. The manifest records the exact native formatter version/digest/profile/options.

The producer pairs exact non-comment code tokens before and after formatting and rejects token spelling/order changes. Compiler name/reference identities map onto the formatted UTF-8 positions. Astro renders those bytes and inserts anchors, using responsive soft wrapping for narrow screens; there is no second Jai syntax formatter. This policy does not rewrite ordinary Markdown paragraphs or code examples.

Generated lint examples use explicit versioned fence metadata to produce typed `example_group` nodes. A source owns at most one diagnostic/output result in the same Markdown container; corrected examples are separate groups. Unknown metadata versions/fields, orphan results and duplicate results fail extraction. Ordinary unmarked code blocks and authored “Example:” paragraphs are independent content.

## Consumer admission

The pinned schema generator derives TypeScript declarations and standalone runtime validators from the same Rust authority. `tools/generators/generate_doc_site.py --check` compares generated outputs without rewriting checked-in files. It requires explicitly installed builder dependencies; it never installs them itself.

The Astro loader checks versions, payload shapes, inventories, lengths/hashes, portable paths, route/fragment references, capabilities, token spans and asset provenance before rendering. Components receive admitted typed data and use normal escaping. A malformed export fails before replacing an existing site; compatibility is not inferred from a few familiar fields.

The builder stages a complete static tree, checks emitted HTML and local links/fragments with an independent parser, then replaces only Jacket-owned output. Example JSON, compact navigation data and search data use generated validators in the browser as well. Rendering or exporting does not execute examples by default; execution requires the explicit validation operation or a browser Run action.

## How to change it

Change compiler facts in `doc_comments/model.rs` and strict Jacket wire DTOs/tests together. Change the site contract in `export/contract.rs`, then regenerate the JSON Schema, TypeScript declarations and every standalone validator. Update producer mapping, loader/graph checks, components and malformed-input regressions as one change. A change incompatible with either public contract requires an explicit version decision; do not add permissive defaults to make an old payload pass.

Extend declaration layout in `stdlib/Extensions/Jai_Format/declaration.jai` and its existing formatter planner. Keep semantic token extraction in the compiler; `website/docs/src/components/DeclarationCode.astro` only presents formatter-authored lines and anchors. Extend Markdown policy in the shared document parser and production components, then review the complete pinned CommonMark/extension ledger; expected output must not be regenerated from actual output.

## Configuration and dependencies

`JAIC_STDLIB` and `-import_dir` control compiler library roots. Jacket's `[doc]` configuration and CLI flags select the library, guides, publication origin/base, runtime assets and explicit example validation. `JACKET_DOCS_BUILDER` selects a compatible source builder; `JACKET_DOC_EXPORT` selects an existing export for direct Astro work.

Dependencies are the compiler parser/resolver, Jacket's typed producer and Serde/schema derivation, the pinned Markdown parser, and the explicitly installed Astro/TypeScript/runtime-validation toolchain. See [doc comments](doc-comments.md), [jacket doc](../tools/jacket-doc.md) and [Astro development and packaging](../tools/astro-docs.md) for setup and acceptance procedures. Local conformance receipts do not establish final-head CI, browser parity, performance or deployed-site acceptance.

Runnable example asset IDs hash the complete typed example payload with its ID left empty. Different timeouts, templates, source links or other payload facts therefore produce different assets even when the executable text matches. Explicit example validation deduplicates only exact executable text and timeout; different budgets retain separate checks without a truncated hash key. The builder still rejects an ID collision with unequal payload bytes.

Asset provenance is a generated enum: configured logo, configured social image, browser runtime, or generated publication metadata. The owner tags bytes when it gathers them; the export does not infer provenance from file extensions. Browser runtime and editable-example capabilities occur together and require exactly the declared engine and Wasm assets with their expected media types and runtime provenance. Both Rust and the importing builder reject incomplete, extra or misclassified runtime assets before rendering.

The compiler raw model has one declaration-token authority: each item’s versioned `definitions` inventory. The former canonical `declaration_tokens` tuple and `decl_refs` copies are removed; each definition carries its own ordered tokens, source bounds and references. Extend that definition structure and its strict importer together, including conditional variants.
