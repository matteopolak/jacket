# Environment

`Extensions/Environment` reads the process environment, so modules and tests stop declaring their own `getenv`.

```jai
#import "Extensions/Environment";

url := environment_variable("DATABASE_URL");
if !url url = "postgres://localhost/app";
if has_environment_variable("NO_COLOR") colors = false;
```

## How it works

`environment_variable(name)` returns a copy of the value in temporary storage, or `""` when it is not set. `has_environment_variable(name)` tells an unset variable from one set to the empty string. On Windows it calls `_wgetenv` with the name converted to UTF-16 and converts the value back, so non-ASCII values survive. Elsewhere, including WASI, it calls libc's `getenv`.

## How to change it

Keep it to reading. Setting variables changes state other threads may be reading, so a caller that needs to set one should do it at startup through its platform module (`POSIX.setenv`, for example).

`Extensions/Io` keeps its own reader, `env_text`, which fills a caller's buffer without allocating.

## Configuration

None.

## Dependencies

`Basic`, plus `Windows` and `Windows_Utf8` on Windows. Users: `Args`, `Postgres` (connection defaults, `PGPASSFILE`, the typed-query settings, managed binaries' cache folder) and the Postgres tests and tools.
