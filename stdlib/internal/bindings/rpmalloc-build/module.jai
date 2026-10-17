// Independent packaging default. The public import parameter list
// does not declare FIRST_CLASS_HEAPS. configure_rpmalloc.py supplies a build's
// explicit selection without extending the public module parameters.
FIRST_CLASS_HEAPS :: false;

// PRETTY_PRINT is likewise undeclared in the public raw parameter list.
// Preserve our actual native diagnostic output as an explicit private policy.
PRETTY_PRINT :: true;
