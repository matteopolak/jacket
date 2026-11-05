# LSP protocol boundary

The language server uses pinned `lsp-server` messages, IDs, request queues and I/O, with `lsp-types` schemas for standard wire data and method names. Jai compiler facts and editor behavior remain independent of those transport types.

## How it works

`position.rs` re-exports the library's UTF-16 `Position` and `Range`; document changes use `TextDocumentContentChangeEvent`. The protocol modules decode standard request parameters and client capabilities, construct typed server capabilities, and translate Jai's richer semantic enums at the JSON boundary. Jai extension payloads remain local types.

Two gaps in `lsp-types` 0.97 need explicit adapters: its server capability struct lacks `typeHierarchyProvider`, and its workspace client diagnostic field uses `diagnostic` where the protocol specifies `diagnostics`. Capability tests compare the entire advertised JSON object with the previous contract. Initialization requires standard capabilities. Code actions require the standard context and diagnostic fields.

Full and range semantic-token responses use `SemanticTokens`; results become the library response’s owned JSON value. The bounded previous-result table stores those same token structs. Delta edits keep a small integer-span serializer: the protocol permits edits within a token, but the pinned library's `SemanticTokensEdit.data` requires complete five-integer token groups. Preserving a one-integer edit avoids widening existing deltas or allocating another flattened array. A regression checks this exact case against the [standard integer-array shape](https://github.com/microsoft/vscode-languageserver-node/blob/main/types/src/main.ts).

Local file identity is still `DocumentUri`: standard `Uri` parsing does not establish the server's local-file policy, URI length limit, platform path rules, or overlay identity. Decode standard URIs first, then apply that policy at the boundary.

## Diagnostic availability

A failed semantic or hint guard is unavailable diagnostics, not an empty clean report. Document and workspace pulls return standard `ServerCancelled` with `DiagnosticServerCancellationData { retriggerRequest: false }`, as permitted by the [pull-diagnostic cancellation contract](https://microsoft.github.io/language-server-protocol/specifications/lsp/3.17/specification/#diagnosticServerCancellationData). A later client trigger can request a fresh result; the server does not retry unstable evidence inside the request.

Push omits an unavailable document so the client retains its prior accepted diagnostics. A workspace pull fails if an open document's report is unavailable. Closed-document empty reports and explicitly syntax-only sessions keep their separate existing behavior. The optional cancellation data uses the pinned library schema and the normal bounded error encoder.

## Bounded transport

Normal requests, notifications and responses use `lsp_server::Message`; IDs follow its `i32`/string contract. Wide integers and floating-point spellings are rejected. `ReqQueue` owns pending requests and cancellations; unknown/completed cancellations are ignored. Outstanding refresh requests are coalesced by method until their responses arrive.

Native stdio uses `Connection` channels and `Message::read/write`. A bounded `BufRead` adapter checks cumulative headers and declared body length before the library allocates the body. Invalid framing, excessive length and truncation terminate the stream. A complete malformed body receives a standard parse/invalid-request error and can be followed by another frame. Reserved-field validation rejects duplicates, contradictory envelopes and invalid IDs before dispatch; payload-bearing parser errors are never logged.

Valid peer responses with an explicit null ID are consumed without replying: they cannot identify a pending server request. This edge requires a complete versioned response, exactly one result or well-formed error, and no request fields or duplicate reserved fields. Missing IDs and null-ID requests remain invalid.

The library cannot represent an unidentified response ID. One typed error-only adapter writes `id: null` for that normative JSON-RPC edge. Valid identified errors use ordinary library responses. Ordered bounded routing controls keep rejected bodies in the same arrival/dispatch/output order as normal Connection messages.

Normal outputs are counted through a bounded serializer before enqueue, then framed by the library writer. Native batches are limited by message count and decoded body bytes; each response is emitted before dispatching the next request. The input and output queues each have bounded backpressure. The writer owns destruction of sent messages; allocation and writer-backpressure measurements must include that work rather than assume a separate dropper thread. A broken output pipe wakes the semantic owner even when stdin stays open. After shutdown, publication/compile work stops and the native host waits at most30 seconds for exit. Shutdown flush waits are bounded, and an abort does not join a reader blocked on an open stdin pipe.

Native and browser share a nonblocking lifecycle: initialize, await a valid `initialized` object, running, shutdown, exit. The library’s blocking convenience lifecycle helpers cannot drive the synchronous browser bridge or preserve the shared bounded-output path. Both hosts instead dispatch the same library Messages through this shared state machine.

## Changing the boundary

Use library request/notification method constants and existing parameter/result types before defining another wire schema. Keep compiler IDs and domain enums out of protocol types; convert them once in the wire modules. New Jai extension types belong beside the handlers that own them.

When updating either library, review the two capability adapters and run protocol, robustness, URI, stdio, and standard-schema tests. Preserve useful nonempty answers as well as JSON shape. An adapter should state the specific missing field or supported compatibility behavior rather than duplicate a complete standard schema.

## Configuration and dependencies

Both dependencies are exact-pinned in `crates/jai-language-server/Cargo.toml` and recorded in the workspace lockfile. It follows the repository's dependency age gate. Native stdio and the browser bridge share the protocol boundary; their filesystem and scheduling implementations remain separate.

See [checked snapshots](lsp-snapshots.md) for semantic authority and [latency measurements](../tools/lsp-latency.md) for runtime measurement rules.

Refresh requests coalesce by method while the client acknowledges one outstanding request. A later change marks that method dirty; its acknowledgement issues one follow-up refresh so a configuration save cannot be lost behind an earlier pull. Follow-ups use new library request IDs and the same output limits, and stop after shutdown. Extend the method admission and tests together when adding refresh providers.
