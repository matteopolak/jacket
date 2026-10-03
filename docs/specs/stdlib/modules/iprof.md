# Iprof

## Module parameters

- `IMPORT_MODE := Import_Mode.METAPROGRAM`
- `MANUAL_MODE := false`
- `USE_GRAPHICS := true`
- `DO_HISTORY := true`

```jai
Import_Mode :: enum u8 {
    METAPROGRAM :: 0;
    CLIENT      :: 1;
}
```
## Public declaration inventory

Visible public declarations and module-conditional variants are listed below. Procedure bodies and aggregate initializers are omitted; complete public type members are listed separately.

- [USE_GRAPHICS] `set_cursor_screen_coordinates :: (x: s32, y: s32)`
- [USE_GRAPHICS] `Graph_Scaling :: struct`
- [USE_GRAPHICS] `Config :: struct`
- [USE_GRAPHICS] `draw :: (x: float, y: float, full_width: float, requested_height: float, config: *Config)`
- [USE_GRAPHICS] `draw_graph :: (sx: float, sy: float, requested_width: float, requested_height: float, config: *Config)`
- [USE_GRAPHICS] `graph_select_sample :: ()`
- [USE_GRAPHICS] `get_current_graph_scaling :: () -> *Graph_Scaling`
- `get_plugin :: () -> *Metaprogram_Plugin`
- `before_intercept :: (p: *Metaprogram_Plugin, flags: *Intercept_Flags)`
- `add_source :: (p: *Metaprogram_Plugin)`
- `message :: (p: *Metaprogram_Plugin, message: *Message)`
- `finish :: (p: *Metaprogram_Plugin)`
- `shutdown :: (p: *Metaprogram_Plugin)`
- `Zone_Info :: struct`
- `TO_INSERT :: #string DONE`
- `prepend_main :: #code context._Iprof.Prepend_To_Main()`
- `add_hook :: (w: Workspace, iprof_hook_name: string, iprof_hook_args: string, user_hook_name: string, user_hook_args: string)`
- `init :: (p: *Metaprogram_Plugin, options: [] string) -> bool`
- `log_help :: (p: *Metaprogram_Plugin)`
- `HELP_STRING :: #string DONE`
- `log_text_report :: (report: *Report, min_self_time := 0., min_hier_time := 0.)`
- `save_csv_report :: (report: *Report, filename: string)`
- `MAX_DEPTH :: 100`
- `MAX_HASH_SIZE :: 65536`
- `INIT_HASH_SIZE :: 256`
- [DO_HISTORY] `Zone_History :: struct`
- `Zone :: struct`
- `Zone_Profiling_Data :: struct`
- `zone_begin :: inline (zone_index: s64) #expand`
- `zone_end :: inline ()`
- `NUM_VALUES :: 4`
- `NUM_TITLE :: 2`
- `NUM_HEADER :: NUM_VALUES+1`
- `Report_Row :: struct`
- `Report :: struct`
- `Report_Mode :: enum u8`
- `NUM_FRAME_SLOTS :: 128`
- `MAX_PROFILING_ZONES_FOR_MANUAL_MODE :: 2048`
- `NUM_SMOOTHING_SLOTS :: 3`
- `NUM_THROWAWAY_UPDATES :: 3`
- `INT_ZERO_THRESHHOLD :: 0.25`
- `Prof_init_highlevel :: ()`
- `set_report_mode :: (desired: Report_Mode)`
- `update :: (record_data: bool)`
- `create_report :: () -> *Report`
- `Stack_Push :: (zone: *Zone) -> *Zone_Profiling_Data`
- `traverse :: (func: (z: *Zone_Profiling_Data))`
- `move_cursor :: (num: s64)`
- `set_cursor :: (num: s64)`
- `select :: ()`
- `select_parent :: ()`
- `set_frame :: (num: s64)`
- `move_frame :: (delta: s64)`
- `set_smoothing :: (x: s64)`
- `init_runtime :: ()`
- `hash :: (z: *Zone, s: *Zone_Profiling_Data) -> u64`
- `insert_node :: (q: *Zone_Profiling_Data) #no_aoc`
- `count_recursion_depth :: (stack: *Zone_Profiling_Data, zone: *Zone) -> s32`
- `createStackNode :: (zone: *Zone, parent: *Zone_Profiling_Data) -> *Zone_Profiling_Data`
- `init_zone_hash :: (size: s64)`
- `History_Scalar :: struct`
- `Profile_Tracker_Data_Record :: struct`
- `FRAME_TIME_INITIAL :: 0.001`
- `clear :: (using s: *History_Scalar)`
- `update :: (using s: *History_Scalar, new_value: float64, k_array: [] float64)`
- `eternity_set :: (using s: *History_Scalar, new_value: float64)`
- `get_value :: (using s: *History_Scalar) -> float64`
- `propagate_stack :: (c: *Zone_Profiling_Data)`
- `clear_stack :: (using c: *Zone_Profiling_Data)`
- `sum_times :: (c: *Zone_Profiling_Data)`
- `update_history :: (c: *Zone_Profiling_Data)`
- `SPEEDSTEP_DETECTION_RATIO : float64 : 0.08`
- `propagate_to_zone :: (c: *Zone_Profiling_Data)`
- `propagate_expanded :: (c: *Zone_Profiling_Data)`
- `compute_heat :: (variance: float64, value: float64) -> float64`
- `report_compare_slot :: (p: Report_Row, q: Report_Row, slot: s64) -> s64`
- `report_expand_compare :: (a: Report_Row, b: Report_Row) -> s64`
- [USE_GRAPHICS] `GraphLocation :: struct`
- [IMPORT_MODE == .CLIENT] `Automatic_Zone :: Automatic_Zone_Guarded`
- [IMPORT_MODE != .CLIENT] `Automatic_Zone :: (index: s64) #expand`
- [IMPORT_MODE == .CLIENT] `Automatic_Zone_Guarded :: (index: s64) #expand`
- [IMPORT_MODE == .CLIENT] `Prepend_To_Main :: () #expand`

- `caches : [] *Zone_Profiling_Data`
- `current_graph_window_selected_sample := -1`
- `cursor: s64`
- `display_frame : s64`
- `displayed_quantity := Report_Mode.SELF_TIME`
- `dummy: Zone_Profiling_Data`
- `dummy2: Zone_Profiling_Data`
- `frame_time: History_Scalar`
- `graph_bigness_t_target: float`
- `history_index : s64`
- `injected_final_profiling_code := false`
- `instrumented_entry_point := false`
- `integer_timestamps_per_second: History_Scalar`
- `last_integer_timestamp: s64`
- `last_update_time: float64`
- `next_zone_under_cursor: *Zone`
- `num_active_zones: s64`
- `paused := false`
- `precomputed_factors : [NUM_SMOOTHING_SLOTS] float64`
- `prof_sum: s64`
- `slot: s64 = 1`
- `speedstep_warning: bool`
- `stack: *Zone_Profiling_Data`
- `times_to_reach_90_percent: [NUM_SMOOTHING_SLOTS] float64`
- `timestamps_to_seconds: float64`
- `update_cursor := false`
- `update_index: s64`
- `zone_hash: [..] *Zone_Profiling_Data`
- `zone_hash_count: s64 = 0`
- `zone_hash_mask: u64 = 0`
- `zone_infos: [..] Zone_Info`
- `zone_under_cursor: *Zone`
- `zones: [MAX_PROFILING_ZONES_FOR_MANUAL_MODE] Zone`
- `zones: [] Zone`

## Public type members

#### `Graph_Scaling` (struct)

- `target:  s64`
- `current: float`
- `ease:    float`
- `output_factor: float64`
- `SCALE_MAX :: 16`

#### `GraphLocation` (struct)

- `x0, y0: float`
- `sx, sy: float`

#### `Report_Mode` (enum u8)

- `SELF_TIME`
- `HIERARCHICAL_TIME`
- `CALL_COUNT`
- `CALL_GRAPH`

#### `Config` (struct)

- `line_spacing          : s32 = 8`
- `font_character_height : s32 = 8`
- `draw_text:      (x: float, y: float, str: string, color: Vector4)                    = dummy_draw_text`
- `text_width:     (s: string) -> float                                                 = dummy_text_width`
- `draw_rectangle: (x0: float, y0: float, x1: float, y1: float, color: Vector4)         = dummy_draw_rectangle`
- `draw_line:      (p0: Vector2, p1: Vector2, p2: Vector2, p3: Vector2, color: Vector4) = dummy_draw_line`
- `graph_begin:    (x: float, y: float, width: float, height: float)                    = dummy_graph_begin`
- `graph_end:      (x: float, y: float, width: float, height: float)                    = dummy_graph_end`
- `limit_name_column_character_length := false`
- `zone_time_precision_digits: s32 = 2`
- `title_background_color := Vector4.{.05, .05, .05, .97}`
- `title_text_color       := Vector4.{.7, .7, .7, 1}`
- `zone_text_color_cold   := Vector4.{.8, .15, .15, 1}`
- `zone_text_color_hot    := Vector4.{1, .9, .9, 1}`
- `header_color_unhilit := Vector4.{.6, .6, .6, 1}`
- `header_color_hilit   := Vector4.{1, 1, .7, 1}`
- `bar_color_a := Vector4.{.1, 0, .03, .93}`
- `bar_color_b := Vector4.{.14, .05, .06, .93}`
- `selected_bar_whiten_amount := 0.35`
- `graph_zone_under_cursor_text_color       := Vector4.{.8, 1, 1, 1}`
- `graph_zone_under_cursor_background_color := Vector4.{0, 0, .1, .9}`
- `graph_current_time_marker_color          := Vector4.{1, 1, .9, 1}`
- `graph_current_time_history_color         := Vector4.{1, .3, .3, .75}`
- `graph_time_cursor_color                  := Vector4.{1, 1, .9, .5}`
- `selected_graph_color                     := Vector4.{1, 1, 1, 1}`
- `text_y_offset : s32 = 0`
- `graph_colors: [] Vector4`
- `user_data: *void`

#### `Zone_Info` (struct)

- `name:      string`
- `load:      *Message_File`
- `line:      s64`

#### `Zone_History` (struct)

- `values: [NUM_FRAME_SLOTS] float32`

#### `Zone` (struct)

- `name: string`
- `hash: u64`
- `filename: string`
- `full_path: string`
- `line:     s64`
- `highlevel:    *Report_Row`
- [DO_HISTORY] `history_self: Zone_History`
- [DO_HISTORY] `history_hier: Zone_History`
- [DO_HISTORY] `history_calls: Zone_History`
- `visited := false`
- `initted := false`

#### `Zone_Profiling_Data` (struct)

- `t_self_start:     s64`
- `total_self_ticks: s64`
- `total_hier_ticks: s64`
- `total_entry_count: u32`
- `parent: *Zone_Profiling_Data`
- `zone:   *Zone`
- `recursion_depth:   s32`
- `profile_tracker_data_record:         *Profile_Tracker_Data_Record`

#### `Report_Row` (struct)

- `indent:     s64`
- `name:       string`
- `number:     s64`
- `prefix:     u8`
- `value_flag: s64`
- `self_time:  float64`
- `hier_time:  float64`
- `calls:      float64`
- `#overlay (self_time) array_aliased_values: [3] float64`
- `heat:       float64`
- `zone: *Zone`

#### `Report` (struct)

- `title:         string`
- `title_right:   string`
- `header:  [NUM_HEADER] string`
- `hilit_header: s32 = 0`
- `records: [..] Report_Row`
- `hilight: s64`

#### `History_Scalar` (struct)

- `values:    [NUM_SMOOTHING_SLOTS] float64`
- `variances: [NUM_SMOOTHING_SLOTS] float64`
- `history:    [NUM_FRAME_SLOTS] float`

#### `Profile_Tracker_Data_Record` (struct)

- `self_time: History_Scalar`
- `hierarchical_time: History_Scalar`
- `entry_count: History_Scalar`
- `max_recursion: s64`

## Behavioral contract and constraints

IProf records named profiling zones and produces text or CSV reports. Rows with zero calls are omitted; in manual mode, uninitialized zones are also omitted. Timing/counter accuracy depends on the platform timer and explicit zone initialization. The module does not guarantee thread-safe aggregation.

## Report, history, and instrumentation behavior

`log_text_report` emits self time and hierarchical time in milliseconds plus call counts. Positive `min_self_time` and `min_hier_time` thresholds are applied independently: a row is excluded when it is below either enabled threshold. Non-positive thresholds are disabled. Text spacing and numeric presentation are for human reading and are not a stable machine-readable format.

`save_csv_report` writes a header row with the columns `name`, `self (ms)`, `hier (ms)`, and `calls`, followed by report rows. Time values are in milliseconds. The first informational line identifies the invoking command. Numeric precision and the informational line's exact text are not ABI guarantees. Zone names are emitted as supplied; escaping of delimiter characters inside names is not guaranteed. A file-write failure is reported through the module's logging path; the procedure has no success result.

The three history smoothing slots target approximately 90% response after 0.1, 0.8, and 2.5 seconds; the selected slot defaults to the middle value and is clamped to the available range. The initial three updates seed the smoothed values directly from observations. With `DO_HISTORY` enabled, history retains 128 frame samples. Calling `update(false)` closes and calibrates current timing data but does not record a frame-history sample or advance the recorded-frame index. Disabling `DO_HISTORY` removes the per-zone history fields and graph history storage from that configuration.

The `-csv filename` option selects a CSV destination, `-min_size integer` instruments only procedures meeting the minimum expression count, and `-modules` includes modules as well as the main program. Missing arguments, malformed integers, trailing junk in an integer argument, and unknown options cause initialization to report failure. CSV output is generated at runtime and overwrites the selected destination. `TO_INSERT` is compiler-inserted client setup that makes the instrumentation import available under the `_Iprof` alias; its literal generated text is not a compatibility surface. `prepend_main` arranges for runtime initialization before the program's main work. The exact help-string wording is informational.

The metaprogram adds two wrappers after it instruments the entry point: `iprof_per_frame_hook :: ()` invokes the configured per-frame callback with no arguments, and `iprof_editor_hook :: (filename: string, line: s64)` invokes the configured editor callback with `(filename, line)`. Both callbacks are optional; an empty callback name leaves the wrapper without a client call. The exported `add_hook` helper has the same behavior for caller-supplied wrapper name/signature and callback name/argument text: the signature text becomes the wrapper parameter list, and the callback argument text is forwarded verbatim as that call's arguments. The hook names above are the names requested by Iprof for runtime integration; the user callback names are supplied by the plugin configuration.

The configured `-min_size` is a compile-time instrumentation filter measured by the procedure body's expression count. The `min_self_time` and `min_hier_time` arguments to `log_text_report` are separate report filters: they compare against `Report_Row.self_time` and `.hier_time`, whose values are milliseconds. Nonpositive report thresholds disable their respective filters.

There is an exposure gap for the predicate and hook-name configuration mentioned above. `get_plugin` publicly returns `*Compiler.Metaprogram_Plugin`, whose public record has no `should_instrument`, `per_frame_hook_name`, or `editor_hook_name` fields. Iprof internally allocates a larger derived record with those fields, but that derived record is declared at module scope and is not part of the importer-visible contract. Thus the callback type (`(body: *Compiler.Code_Procedure_Body) -> bool`) and names are observable implementation requirements, but callers cannot configure them through the exported return type under the published API. No public cast, setter, or alternate returned type is supplied to bridge that gap; implementers must preserve the public return type and should treat the custom configuration path as unavailable unless another public declaration establishes it.

`IMPORT_MODE` selects separate interfaces. In `METAPROGRAM` mode, `get_plugin` registers the init, interception, source insertion, message, finish, shutdown, and help callbacks. `init` accepts `-csv path`, `-min_size n` (default `30`), and `-modules`; unknown options, missing values, malformed integers, or trailing non-space input return false. Iprof's internal derived plugin record has a `should_instrument: (*Compiler.Code_Procedure_Body) -> bool` field; when set, it further filters eligible non-entry-point procedures. That field is not available through the exported `get_plugin` return type, as clarified above. The plugin skips polymorphic, compiler-generated, compile-time-only, quick, type-only, macro, already-modified, and no-context procedures. The entry point is considered regardless of size or predicate and receives the initialization prefix before its profiling zone, provided it passes the unconditional procedure-flag and context eligibility checks. Other main-program procedures must meet the minimum expression count and predicate. Other imported modules are skipped unless `-modules` is set; Iprof, preload/runtime-support modules, Pool, and the Basic procedures named `temporary_allocator_proc` and `IsScalar` are excluded. Procedures declared in Basic's `module.jai`, and Thread procedures, use the guarded zone macro. The injected zone exit is deferred, so it runs on normal scope exit and early returns. At the `TYPECHECKED_ALL_WE_CAN` phase, once an entry point was instrumented, the plugin adds zone-name initialization and the per-frame/editor wrappers described above. The plugin's finish callback is empty; shutdown releases the plugin instance.

In `CLIENT` mode, `Automatic_Zone(index)` aliases `Automatic_Zone_Guarded(index)`. The guarded macro begins/ends the indexed zone unless the current context says execution is already inside Iprof's runtime; the end is deferred. In metaprogram mode `Automatic_Zone` is an empty expansion. `Automatic_Zone_Guarded` and runtime profiling functions are part of the client runtime interface. `USE_GRAPHICS=false` omits the graph/UI declarations (`Config`, drawing helpers, screen-coordinate setter, and graph selection/scaling APIs); reporting and runtime collection remain available. The callback fields in `Config` default to diagnostic stubs: each drawing callback logs that it is unset, and `text_width` logs and returns zero. A working renderer must supply all callbacks it uses and appropriate line/font metrics.

History scalars smooth both the value and its second moment with `old*k + sample*(1-k)` for each configured response factor; the resulting reported variance is the smoothed second moment minus the squared smoothed value. `clear` zeros only the smoothed values and second moments, leaving the frame ring untouched. `eternity_set` assigns every smoothing slot to the new sample and every second-moment slot to its square. Both `update` and `eternity_set` write the current ring-buffer cell when history is enabled but do not advance the ring index. The first three runtime frame updates use `eternity_set` rather than smoothing; after that the three response slots target 90% response over 0.1, 0.8, and 2.5 seconds. `SPEEDSTEP_DETECTION_RATIO` is `0.08`; the runtime compares the standard-deviation-to-absolute-mean ratio of the middle timestamp-rate slot to this threshold. `clear_stack` clears only the node's current self ticks, hierarchical ticks, and entry count; it does not clear tracker history or recursion maxima.

The public node helpers expose these effects: `createStackNode` allocates a persistent profiling node from the initialized runtime pool, records its zone and parent, and derives recursion depth from matching zones in the parent chain. `hash(zone, parent)` returns the zone's declared hash plus the parent's pointer value converted to `u64`. `init_zone_hash(size)` replaces the table contents with empty sentinel entries and resets its count; it does not preserve or rehash prior nodes. Its size must be a positive power of two no larger than `MAX_HASH_SIZE` for the mask-based table to operate correctly, although the helper asserts only the upper bound. `insert_node` inserts into an initialized table and increments its count; it does not deduplicate matching zone/parent pairs. `Stack_Push` performs the reuse/deduplication behavior used by normal profiling. `traverse` visits each non-sentinel table entry in physical table order. Mutating, resizing, or reinitializing the table from its callback is not a supported traversal guarantee. These runtime structures are not promised thread-safe.

`update(record_data)` first propagates each node's self ticks to each distinct zone on that node's stack chain, including its own zone, which produces inclusive hierarchical totals without multiplying recursive ancestors. It computes a timer calibration on the first update from the sum of self ticks (using `1` when the sum is zero); later updates use the CPU-counter delta. It then updates zone histories when `record_data` is true and clears the three current counters on every node before returning. `sum_times` sums all visited self ticks. `update_history` updates or creates each node's tracker record, tracks maximum recursion, converts self/hierarchical ticks to seconds, and records entry counts; on the first three updates those values seed all smoothing slots. `propagate_to_zone` transfers selected self/hierarchical seconds to report rows in milliseconds and transfers call counts. `propagate_expanded` produces the selected call-graph rows for the expanded zone, its child/parent relationships, and prefixes indicating active callers. `compute_heat` subtracts the squared value from the supplied second moment, clamps negative variance to zero, and returns a relative standard-deviation score clamped to 0–1 (zero for values near zero). `report_compare_slot` orders by the chosen metric descending, then by name length and lexical name for stable ties. `report_expand_compare` places the parent-context group before child rows and the expanded-zone row; parent-context rows are ordered by increasing hierarchical time, while child and expanded rows are ordered by decreasing hierarchical time. Equal call-graph metrics compare equal.

`create_report` allocates the report and row array from `temporary_allocator`; callers must consume it before that allocator is reset. Row names and zone pointers refer to live zone storage. It creates one row per zone for ordinary modes and three candidate rows per zone for call-graph mode, then removes zero-call call-graph rows and sorts them. It updates the zones' `highlevel` pointers to the temporary rows. The title presents average milliseconds per frame and the corresponding FPS; the right title is `current frame` or a selected frame offset into the past. Header slots 1–3 identify self time, hierarchical time, and call count (slot 0 is left blank). `move_cursor` and `set_cursor` change the stored row index without immediate clamping; report creation clamps it to the current row range. `select` expands the highlighted zone and switches to call-graph mode. `select_parent` requests cursor remapping and chooses among the non-current rows before the expanded-zone row in the current sorted report; the module does not filter those candidates to prove they are only parent rows, so a particular parent identity is not guaranteed when multiple candidates appear. `set_frame` clamps the displayed history offset into `0..NUM_FRAME_SLOTS-1`; `move_frame(delta)` moves the offset in the opposite sign direction (`delta < 0` moves into the past). `graph_select_sample` uses the selected sample index relative to the history ring's current index, adds one ring length if the result is negative, and expands the zone under the graph cursor when present. `get_current_graph_scaling` returns the self-time scale only in self-time mode and the hierarchical scale otherwise. Graph scaling state starts zero-initialized; on each graph draw, its current scale moves toward the integer target in powers per second at a base rate of 5 (doubled for a target gap greater than one), weighted by the square root of its ease value. Ease rises toward one at 5 per second while the target is unmet and resets to zero at the target; the output factor is `0.5^current`. Graph layout grows from the requested dimensions according to its animated size factor. The module does not define a compatibility requirement for callback-specific pixel widths, visual styling choices, or the exact order of equal hash-table traversal results beyond slot order.
