# Independent import-metadata fixtures

These archives were generated from this definition by official LLVM and GNU
MinGW dlltool. They are metadata inputs, not executable DLLs or copied SDK bodies.
The test checks the declared physical provider and machine. No function execution
or SDK capability is inferred from them.

```def
LIBRARY independently_named_42.dll
EXPORTS
    ordinary_entry
```

Regenerate in an owned T7 output directory, then copy these three files into the
fixture directory:

```sh
llvm-dlltool -m i386:x86-64 -d provider.def -l ms-x64.lib
llvm-dlltool -m arm64 -d provider.def -l ms-arm64.lib
x86_64-w64-mingw32-dlltool -m i386:x86-64 -d provider.def -l gnu-x64.a
```

Actual Windows SDK/provider execution is a separate required CI obligation.
