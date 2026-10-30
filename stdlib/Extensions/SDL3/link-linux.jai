// Optional audio/window backends are dlopened by SDL. No development libraries at runtime.
#library,system,link_always "dl"; @native(external)
#library,system,link_always "pthread"; @native(external)
#library,system,link_always "m"; @native(external)
#library,system,link_always "rt"; @native(external)
