# Project_Generator


## Public declaration inventory

Only names visible through the module’s exported scope are listed. Declarations from conditional branches retain their condition label; implementation-local declarations are omitted. Aggregate fields and enum members are recorded in the dedicated type-shape section.

- `Project_Generator : struct`


## Public aggregate shapes and enum values

Each block gives the exported type declaration with its member fields or enum values. Conditional declarations with distinct shapes are shown as separate variants; platform labels describe availability, not source organization.

### `Project_Generator`

```jai
Project_Generator : struct {

    project_name := "";
    output_directory := "";
    debug_command := "";
    working_directory := "";
    run_arguments := "";
    build_cmd := #run get_path_of_running_executable();
    build_args := "";

    module_path := #filepath;

    project_files : [..]string;

reset :: ();

add_file :: (file: string, take_ownership: bool);

add_file :: (path: string, file: string);

process_message :: (message: *Message);

build :: (file: string);

intercept_messages :: (w: Workspace);
}
```
## Behavioral contract

`Project_Generator` collects source-file paths from compiler `FILE` messages (excluding files added from strings) and, at `COMPLETE`, writes four Visual Studio artifacts in the current working directory: a solution, a C++ project, its filters file and per-user debugger settings. Identical existing contents are left untouched. The generated project is a Visual Studio Release/x64 scaffold; its build step invokes `build_cmd` plus `build_args`, and its debugger settings use `debug_command`, `working_directory` and `run_arguments`. It is project scaffolding rather than a general cross-platform build system.

### Defaults and field behavior

- `project_name`, `output_directory`, `debug_command`, `working_directory`, `run_arguments`, and `build_args` default to empty strings. At the first completion where they are empty, `project_name` becomes the target executable basename without extension, `debug_command` becomes that basename plus `.exe`, and `working_directory` becomes the target output path. A relative working directory is emitted relative to `$(SolutionDir)`.
- `build_cmd` defaults to the running executable's path. `module_path` defaults to the source file path for the module. `project_files` begins empty.
- `output_directory` is present in the public struct but is not read by the generation path; generated files are written relative to the process working directory. No behavior for this field is established.
- Defaults assigned during generation persist in the public fields for later calls. `reset()` clears and frees only the accumulated `project_files`; it does not restore configuration fields to their initial values.

### File collection, ownership and output

`add_file(file, take_ownership)` keeps paths unique. With ownership enabled, the supplied string storage is adopted; when the path is already present the redundant supplied string is freed. With ownership disabled, a new copy is stored only if the path is not already present. `add_file(path, file)` joins the components with `/` and transfers the resulting owned string into the list. `reset` frees every retained path and clears the list.

`process_message` adds the fully qualified filename from ordinary file messages. On completion it adds the module's `../editor_support/msvc/jai.natvis` path, converts separators for Visual Studio, uppercases a Windows drive letter, makes paths relative to the current working directory when they share that prefix, sorts them, then emits the four project files. Paths with `..` are not converted to folder filters; other folder paths become filters. After writing, it resets the collected file list. It does not create the output directory or honor `output_directory`.

`build(file)` is a convenience interception flow for the root workspace: it suppresses output for the metaprogram itself, creates an executable target workspace, adds the given file, and runs the message interception flow. For an existing metaprogram/interceptor, call `process_message` for each message or use `intercept_messages` after interception has begun and target source has been added. `intercept_messages` forwards only messages for its workspace and ends interception when `COMPLETE` arrives. Build options and the compiler's message stream determine the target name, output path and collected sources.
