# Catching memory bugs

`jacket lint` follows pointers through your program and reports the ones used after their memory is gone: freed blocks, reset temporary storage, returned stack frames and arrays that moved when they grew. It also reports allocations nobody frees. This guide shows what it catches and how to read and tune it. The rules themselves are in [jailint](../tools/jailint.md#lifetime-rules).

## What it catches

| Rule | Bug | Default |
| --- | --- | --- |
| `use_after_free` | memory read after it was freed or its temporary storage was reset | deny |
| `double_free` | the same memory freed twice | deny |
| `returned_local_address` | a pointer to a local returned from its procedure | deny |
| `pointer_into_resizable` | a pointer into a `[..]` array kept across a call that grows it | warn |
| `leaked_allocation`, `leaked_temporary` | an allocation nothing frees | warn |
| `remove_in_for` | a removal inside `for` that skips the next element | warn |
| `temp_storage_escape` | temporary storage kept in a global or returned | allow |
| `pointer_lifetime` | the general rule: also moves, removals and workers | allow |

## Temporary storage that outlives a reset

A helper returns an error whose text it built with `tprint`. The caller resets temporary storage and then prints the error:

```jai
#import "Basic";

Err :: struct { message: string; }

check :: (n: int) -> Err { return .{ message = tprint("bad %", n) }; }

main :: () {
    e := check(3);
    reset_temporary_storage();
    print("%\n", e.message);
}
```

```diagnostic
error[use_after_free]: `e.message` is used after the call above freed it
  --> example.jai:10:18
   |
 8 |     e := check(3);
   |          -------- storage derived here
 9 |     reset_temporary_storage();
   |     ------------------------- storage invalidated here
10 |     print("%\n", e.message);
   |                  ^^^^^^^^^
   |
   = help: free it after its last use, or give it a new value before using it again
   = note: `use_after_free` is `deny` by default
```

Nothing says that `check` returns temporary storage: the linter read its body. That works through any number of calls, through struct fields and through wrappers. A `destroy :: (n: *Node) { free(n); }` frees its argument as far as the linter is concerned, with no note.

Fix it by printing before the reset, or by building the message with `sprint` and freeing it when you are done.

## Addresses of locals

```jai
first_char :: (s: string) -> *u8 {
    buffer: [16] u8;
    buffer[0] = s[0];
    return *buffer[0];
}
```

```diagnostic
error[returned_local_address]: this returns the address of the local `buffer`, which is gone once the procedure returns
  --> example.jai:4:12
   |
 4 |     return *buffer[0];
   |            ^^^^^^^^^^
   |            ---------- stack storage borrowed here
   |
   = help: return the value itself, take the memory from an allocator (`New`, `alloc`), or have the caller pass the place to fill
```

## Allocations nobody frees

```jai
#import "Basic";

greeting :: (name: string) -> string {
    return sprint("hello, %", name);
}

main :: () {
    if greeting("ada").count > 3  print("long\n");
}
```

```diagnostic
warning[leaked_temporary]: the memory `greeting` allocates is only measured and never freed
  --> example.jai:8:8
   |
 8 |     if greeting("ada").count > 3  print("long\n");
   |        ^^^^^^^^^^^^^^^
   |
   = help: keep it in a local and `defer free` it, or use a temporary-storage procedure such as `tprint`
```

`greeting` allocates because `sprint` does. Pass `,, temp` (`greeting("ada",, temp)`) when the result is only looked at.

## Must and strict

By default a lint fires only when the bug happens on **every** path. That keeps it quiet, at the cost of missing bugs that need a particular branch. Strict mode also reports what **may** happen: a free on one branch, an `array_add` that moves the array only when it is full, a foreign procedure that could keep what you pass it.

```toml
# jacket.toml
[lint.lifetimes]
mode = "strict"
```

or `jacket lint --lifetime-mode strict`. Here `names` grows only when `verbose` is set, so the default mode is quiet and strict reports it:

```jai
#import "Basic";

show :: (verbose: bool) {
    names: [..] string;
    array_add(*names, "ada");
    first := *names[0];
    if verbose  array_add(*names, "grace");
    print("%\n", first.*);
}
```

```diagnostic
warning[pointer_into_resizable]: strict mode: `first` may outlive the storage or element it refers to along a bad path; shortest bad path shown (0 other bad paths)
  --> example.jai:8:18
   |
 6 |     first := *names[0];
   |              --------- pointer derived here
 7 |     if verbose  array_add(*names, "grace");
   |        ------- shortest bad path takes this condition as true
   |                 -------------------------- `names` storage invalidated here
 8 |     print("%\n", first.*);
   |                  ^^^^^ pointer used here after its storage was invalidated
```

A strict finding shows one shortest path to the bug, with each branch it takes, and how many other paths lead there. The full finding also lists the calls the effect was found through, down into `array_add`. Strict mode is noisier: use it on code you want held to a higher bar, or run it now and then and read what it says.

## When you still need a note

The linter reads bodies, so a procedure without one has to say what it does. Put the note after the declaration:

```jai
ring_release :: (buffer: *void) #foreign ringlib; @FreesArgument
make_label   :: (n: s32) -> *u8 #foreign ringlib; @Allocates
```

The notes are listed in [declaration notes](../tools/jailint.md#declaration-notes). On a procedure with a body, effects the linter can work out are taken from the body and these notes are ignored. Notes for what a body can't show (an element sort, a logical "this frees part of its argument") still count.

## When it is wrong

Silence one finding with a comment and say why:

```jai
// jailint: allow(pointer_into_resizable)  names was reserved for 16 above
print("%\n", first.*);
```

The linter stays quiet when it can't tell: a pointer passed through a procedure parameter it can't see into, a mutable global or a callback whose target it doesn't know proves nothing in the default mode. It does not look for data races between threads.
