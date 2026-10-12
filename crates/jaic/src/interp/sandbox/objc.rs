//! Stand-ins for the Objective-C runtime, for compile-time code that only registers selectors
//! and looks classes up (Focus's `Objective_C` module fills its selector table and sizes class
//! stubs while it compiles).
//!
//! There is no runtime behind them: a selector or class is a pointer to its own name, interned
//! so the same name always gives the same pointer, and a message send does nothing and answers
//! null. `class` and `superclass` are the two sends whose answers compile-time code checks, so
//! they answer a class (the receiver, and `NSObject`).
use super::*;

/// Every class takes this many bytes; subclasses add none, so stubs sized by the difference
/// from the superclass are empty.
const INSTANCE_SIZE: u64 = 16;

/// The root class every superclass query ends at.
const ROOT_CLASS: &str = "NSObject";

#[derive(Default)]
pub(super) struct Objc {
    selectors: HashMap<String, u64>,
    classes: HashMap<String, u64>,
    /// Stand-in pointer -> its name, for selectors and classes alike (the pointer is the
    /// zero-terminated name, so it is also the answer to `sel_getName`).
    names: HashMap<u64, String>,
}

impl SandboxHost {
    fn objc_intern(&mut self, name: String, class: bool) -> u64 {
        let table = if class {
            &self.objc.classes
        } else {
            &self.objc.selectors
        };
        if let Some(&ptr) = table.get(&name) {
            return ptr;
        }
        let ptr = self.alloc(name.len() + 1);
        if ptr == 0 {
            return 0;
        }
        unsafe { std::ptr::copy_nonoverlapping(name.as_ptr(), ptr as *mut u8, name.len()) };
        let table = if class {
            &mut self.objc.classes
        } else {
            &mut self.objc.selectors
        };
        table.insert(name.clone(), ptr);
        self.objc.names.insert(ptr, name);
        ptr
    }

    pub(super) fn objc_foreign(&mut self, symbol: &str, args: &[u64]) -> Option<u64> {
        let arg = |i: usize| args.get(i).copied().unwrap_or(0);
        Some(match symbol {
            "sel_registerName" | "sel_getUid" => {
                if arg(0) == 0 {
                    return Some(0);
                }
                self.objc_intern(cstr(arg(0)), false)
            }
            "sel_getName" | "class_getName" | "object_getClassName" => {
                if self.objc.names.contains_key(&arg(0)) {
                    arg(0)
                } else {
                    self.objc_intern("nil".to_string(), false)
                }
            }
            "objc_getClass"
            | "objc_lookUpClass"
            | "objc_getRequiredClass"
            | "objc_getMetaClass" => {
                if arg(0) == 0 {
                    return Some(0);
                }
                self.objc_intern(cstr(arg(0)), true)
            }
            "class_getInstanceSize" => INSTANCE_SIZE,
            "class_getSuperclass" => self.objc_superclass(arg(0)),
            "objc_msgSend"
            | "objc_msgSend_stret"
            | "objc_msgSend_fpret"
            | "objc_msgSend_fpret2"
            | "objc_msgSendSuper"
            | "objc_msgSendSuper_stret"
            | "objc_msgSendSuper2" => match self.objc.names.get(&arg(1)).map(String::as_str) {
                Some("class") => arg(0),
                Some("superclass") => self.objc_superclass(arg(0)),
                _ => 0,
            },
            // The frameworks of a compile's target are not on this machine: loading one
            // gives a handle, and anything else is not found.
            "dlopen" => {
                if cstr(arg(0)).starts_with("/System/Library/Frameworks/") {
                    self.objc_intern("framework".to_string(), true)
                } else {
                    0
                }
            }
            "dlclose" => 0,
            _ => return None,
        })
    }

    /// The class above `class`; the root class has none.
    fn objc_superclass(&mut self, class: u64) -> u64 {
        match self.objc.names.get(&class) {
            Some(name) if name != ROOT_CLASS => self.objc_intern(ROOT_CLASS.to_string(), true),
            _ => 0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn call(host: &mut SandboxHost, symbol: &str, args: &[u64]) -> u64 {
        host.objc_foreign(symbol, args)
            .unwrap_or_else(|| panic!("{symbol} is not provided"))
    }

    fn name(text: &str) -> Vec<u8> {
        text.bytes().chain([0]).collect()
    }

    #[test]
    fn selectors_are_interned_and_named() {
        let mut h = SandboxHost::default();
        let (a, b) = (name("init"), name("setFrame:display:"));
        let init = call(&mut h, "sel_registerName", &[a.as_ptr() as u64]);
        assert_ne!(init, 0);
        assert_eq!(init, call(&mut h, "sel_getUid", &[a.as_ptr() as u64]));
        let frame = call(&mut h, "sel_registerName", &[b.as_ptr() as u64]);
        assert_ne!(init, frame);
        assert_eq!(
            cstr(call(&mut h, "sel_getName", &[frame])),
            "setFrame:display:"
        );
    }

    #[test]
    fn classes_are_interned_apart_from_selectors() {
        let mut h = SandboxHost::default();
        let n = name("NSWindow");
        let class = call(&mut h, "objc_getClass", &[n.as_ptr() as u64]);
        assert_ne!(class, 0);
        assert_eq!(
            class,
            call(&mut h, "objc_lookUpClass", &[n.as_ptr() as u64])
        );
        assert_ne!(
            class,
            call(&mut h, "sel_registerName", &[n.as_ptr() as u64])
        );
        assert_eq!(cstr(call(&mut h, "class_getName", &[class])), "NSWindow");
        assert_eq!(call(&mut h, "objc_getClass", &[0]), 0);
    }

    #[test]
    fn sends_answer_null_except_class_and_superclass() {
        let mut h = SandboxHost::default();
        let (c, s, o, w) = (
            name("class"),
            name("superclass"),
            name("NSObject"),
            name("NSWindow"),
        );
        let class_sel = call(&mut h, "sel_registerName", &[c.as_ptr() as u64]);
        let super_sel = call(&mut h, "sel_registerName", &[s.as_ptr() as u64]);
        let other = call(&mut h, "sel_registerName", &[name("init").as_ptr() as u64]);
        let root = call(&mut h, "objc_getClass", &[o.as_ptr() as u64]);
        let window = call(&mut h, "objc_getClass", &[w.as_ptr() as u64]);
        assert_eq!(call(&mut h, "objc_msgSend", &[window, other]), 0);
        assert_eq!(call(&mut h, "objc_msgSend", &[window, class_sel]), window);
        assert_eq!(call(&mut h, "objc_msgSend", &[window, super_sel]), root);
        assert_eq!(call(&mut h, "objc_msgSend", &[root, super_sel]), 0);
        assert_eq!(call(&mut h, "class_getSuperclass", &[window]), root);
        let size = call(&mut h, "class_getInstanceSize", &[window]);
        assert_eq!(size, call(&mut h, "class_getInstanceSize", &[root]));
    }

    #[test]
    fn only_framework_libraries_open() {
        let mut h = SandboxHost::default();
        let framework = name("/System/Library/Frameworks/AppKit.framework/AppKit");
        let handle = call(&mut h, "dlopen", &[framework.as_ptr() as u64, 1]);
        assert_ne!(handle, 0);
        assert_eq!(call(&mut h, "dlclose", &[handle]), 0);
        assert_eq!(
            call(&mut h, "dlopen", &[name("/etc/passwd").as_ptr() as u64]),
            0
        );
    }
}
