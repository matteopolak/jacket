# Objective-C selector check

## What it is

Tests that every Objective-C selector the stdlib sends names a method that exists and takes the arguments it is sent with. A selector is a string (`sel_registerName("setTitle:")`), so the type-checker cannot see a wrong one. A missing `:` (`convertPointToScreen` for `convertPointToScreen:`), a typo, or a method the class does not have all type-check, and then stop the program with "unrecognized selector sent to instance". `crates/jaic-cli/tests/objc_selectors.rs` checks them. CI runs it in the `stdlib-targets` step of `ci.yml` ([continuous integration](continuous-integration.md)).

## How it works

The test reads every `sel_registerName("...")` with a literal selector in `stdlib/`:

1. **Arity, on every host** (`selectors_take_the_arguments_they_are_sent_with`). For a selector passed to a call (`invoke(self, sel_registerName("a:b:"), x, y)`), the number of arguments after it must equal the number of `:` in it. For a selector table entry (`_sel.setTitle_ = sel_registerName("setTitle:")`), the field must be the selector with each `:` written `_`. The test also fails if it finds fewer than 1000 calls or 250 table entries, which would mean the scan no longer recognises the bindings.
2. **Existence, on macOS** (`selectors_exist_in_the_objective_c_runtime`). Each call's receiver gives the class:
   - `objc_getClass("X")` makes it a class method of `X`;
   - `self` makes it an instance method of the type of the procedure's `self` parameter, or of the struct it is declared in. `Lightweight*` views map to `NSOpenGLView` and `NSView`, as `Objective_C.class` maps them, and a polymorphic `self` maps to `NSObject`.

   The test writes a Jai program that loads Foundation, AppKit, QuartzCore, Metal and GameController and asks the runtime about each method: `class_getInstanceMethod`/`class_getClassMethod` for a class, and `protocol_getMethodDescription` through inherited protocols for a protocol such as `MTLDevice`. A class that lacks the method itself still has it when a declared property of it or a superclass has that getter or setter (the attribute string's `G` and `S` names, else `name` and `setName:`; class properties are declared on the metaclass), because some classes implement their properties in a hidden subclass (Metal's descriptors). For an instance method, the subclass `alloc` returns is asked too, which covers class clusters (`NSString`); the object is not initialized or released. It runs the program with `jaic run`, and its output must match `tests/objc-selectors.txt`. Elsewhere, the program only has to type-check for macOS.

A call on a local object (`call(event, sel_registerName("window"))`) gets the arity check but not the existence check, because its class is not written next to it. A selector stored in a variable and sent later is not checked at all, so send selectors as literals at the call.

## How to change it

- When the macOS check reports a method the runtime does not have, fix the binding. If the method only exists on newer macOS than the CI runners, or the class is private, list the line it printed in `tests/objc-selectors.txt` with a `#` reason.
- When it reports a listed line as found, remove the line.

## Configuration

- `cargo test -p jaic-cli --test objc_selectors` (add `--no-default-features` to build without LLVM).
- `tests/objc-selectors.txt`: `missing <class> <instance|class> <selector>` and `unknown <class>` lines; `#` starts a comment.
