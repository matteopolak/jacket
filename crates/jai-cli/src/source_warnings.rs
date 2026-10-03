//! Render checked source warnings only at the final CLI consumer boundary.
pub fn emit(library: &jai_sema::Library) {
    for warning in library.source_warnings() {
        eprintln!("{}", warning.render());
    }
}
