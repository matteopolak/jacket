# Pending distinct modifier witnesses

## What it is

Two original source witnesses distinguish a valid initial `First` baked value from a modifier that changes its dependent formal to the separate nominal type `Other`. They are not registered in the passing checkpoint suite.

## How it works

The preserving-type control should return 21 after its real modifier accepts. The changed-type case must then reach accepted binding normalization and reject the actual `First` value at the original whole `Box(First,initial)` application, retaining the original SourceId.

The current producer rejects `initial:First:21` first with `typed constant annotation requires canonical semantic resolution`. The modifier is not reached. Earlier explicit-cast and invalid contextual-cast attempts also failed before that intended boundary; changing an expected span would not prove dependent nominal rejection.

## How to change it

Finish genuine canonical typed-constant preparation in the retained original arena before a dependent alias consumes it. Preserve the actual `First` TypeId and checked value, then activate both tests together using the existing `record-modifiers.rs` helpers. Do not weaken nominal denial, reroute an upstream diagnostic, or replace the original constant with a guessed scalar representation.

## Configuration

Both witnesses select the explicit lp64 layout through `run_with_options` or `rejected_type_modifier`, because the real Type-valued modifier slots require canonical descriptors. No reference binary or library is executed.

## Dependencies

The retained type/constant cursor, canonical TypeRegistry, checked typed constant pool, actual modifier VM recipe, accepted dependent argument rechecking, and original source diagnostics must all be ready. `witnesses.rs.pending` preserves the exact two test functions for future joint activation.
