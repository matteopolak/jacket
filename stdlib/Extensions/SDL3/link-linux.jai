// Optional audio/window backends are dlopened by SDL. No development libraries at runtime.
#library,system,link_always "dl";
#library,system,link_always "pthread";
#library,system,link_always "m";
#library,system,link_always "rt";
