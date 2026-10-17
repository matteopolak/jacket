// Picks its backend file with `#if OS == { case ...; }`. The chosen file adds a Context member,
// and the module also has a pending `using` of a global whose type has defaults.
#load "settings.jai";

#if OS == {
    case .WINDOWS; #load "backend/context.jai";
    case .LINUX; #load "backend/context.jai";
    case .MACOS; #load "backend/context.jai";
    case; #load "backend/context.jai";
}

backend_ready :: () -> bool {
    return context.backend_state != null && context.backend_state.frames == frame_budget;
}
