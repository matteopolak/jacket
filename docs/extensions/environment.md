# Environment

`Extensions/Environment` reads the process environment, so modules and tests don't declare their own `getenv`.

```jai
#import "Extensions/Environment";

url := environment_variable("DATABASE_URL");
if !url url = "postgres://localhost/app";
if has_environment_variable("NO_COLOR") colors = false;
```

## How it works

- `environment_variable(name)` returns a copy in temporary storage, or `""` when unset.
- `has_environment_variable(name)` tells unset from set-but-empty.
- `output_directory()` is `JACKET_OUTPUT_DIR` when set and not empty, else the `target` folder of the checkout the standard library is in; `output_path(relative)` joins a path below it. Tests and benches that read generated data (the fixture and conformance data `jacket test` builds) use these, so a relocated output directory works.
- Windows goes through `_wgetenv` with UTF-16 conversion, so non-ASCII values survive. Elsewhere it calls `getenv`.

## How to change it

Keep it read-only. Setting variables races with other threads; do it at startup through the platform module (`POSIX.setenv`). `Extensions/Io` has its own allocation-free reader, `env_text`.
