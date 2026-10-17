#module_parameters(MEMORY_DEBUGGER := false, ENABLE_ASSERT := true, REPLACEMENT_INTERFACE: $I/interface Memory_Debugger_Interface = Memory_Debugger_Interface, VISUALIZE_MEMORY_DEBUGGER := true, TEMP_ALLOCATOR_OVERWRITE_MEMORY := false) {
    Memory_Debugger_Interface :: Interface_Definition.Memory_Debugger_Interface;
    #scope_file
    Interface_Definition :: #import "internal/basic-interface";
    #scope_export
}

#load "preamble.jai";
#load "allocation.jai";
#load "memory-debugger.jai";
#load "leak-types.jai";
#load "leak-traces.jai";
#load "leak-reports.jai";
#load "visualization-events.jai";
#load "visualization.jai";
#load "visualization-packets.jai";
#load "arrays.jai";
#load "scalar.jai";
#load "builder.jai";
#load "format-types.jai";
#load "format-integer.jai";
#load "format-support.jai";
#load "numeric-conversion.jai";
#load "format-float.jai";
#load "format-type-names.jai";
#load "format-aggregate.jai";
#load "format-enum.jai";
#load "format.jai";
#load "logging.jai";
#load "stream-helpers.jai";
#load "stack-traces.jai";
#load "reflection.jai";
#load "parsing.jai";
#load "integer128.jai";
#load "apollo.jai";
#load "time-errors.jai";
#load "clock-calendar.jai";
#load "host-clock.jai";
#load "calendar-text.jai";
#load "filetime.jai";
#load "windows-clock.jai";
#load "windows-calendar.jai";
#load "elapsed-time.jai";
#load "temporary.jai";
#if OS == .LINUX || OS == .ANDROID || OS == .MACOS #load "filesystem.jai";
#if OS == .WINDOWS #load "filesystem-windows.jai";
#load "arguments.jai";
