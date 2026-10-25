//! Skipping teardown work at exit.
use jaic::interp::SandboxHost;
use std::cell::RefCell;
use std::rc::Rc;

/// A value the process never tears down: it exits right after, and freeing a large compiler
/// piece by piece takes longer than the exit itself.
pub(super) struct LeakOnExit<T> {
    value: Option<T>,
    before_leak: fn(&mut T),
}

impl<T> LeakOnExit<T> {
    pub(super) fn new(value: T, before_leak: fn(&mut T)) -> Self {
        LeakOnExit {
            value: Some(value),
            before_leak,
        }
    }
}

impl<T> std::ops::Deref for LeakOnExit<T> {
    type Target = T;

    fn deref(&self) -> &T {
        self.value.as_ref().expect("present until dropped")
    }
}

impl<T> std::ops::DerefMut for LeakOnExit<T> {
    fn deref_mut(&mut self) -> &mut T {
        self.value.as_mut().expect("present until dropped")
    }
}

impl<T> Drop for LeakOnExit<T> {
    fn drop(&mut self) {
        // Leaking skips the interpreter's `Drop`, which is what flushes `-profile` counts.
        if let Some(value) = self.value.as_mut() {
            (self.before_leak)(value);
        }
        std::mem::forget(self.value.take());
    }
}

/// Print and drop what the sandbox host has buffered.
pub(super) fn flush_sandbox(host: &Rc<RefCell<SandboxHost>>) {
    use std::io::Write;
    let mut host = host.borrow_mut();
    if host.stdout.is_empty() && host.stderr.is_empty() {
        return;
    }
    jaic::progress::before_output();
    let _ = std::io::stdout().write_all(&std::mem::take(&mut host.stdout));
    let _ = std::io::stderr().write_all(&std::mem::take(&mut host.stderr));
}
