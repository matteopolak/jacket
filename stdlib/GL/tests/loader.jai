#import "Basic";
GL :: #import "GL";

main :: () {
    found := false;

    for member: type_info(GL.GL_Procedures).members {
        if member.name == "glGetStringi" {
            print("glGetStringi kind %, flags %, offset %\n", member.type.type, member.flags, member.offset_in_bytes);
            found = true;
        }
    }

    assert(found);
    address := GL.nsgl_get_proc_address("glGetStringi");

    assert(address != null, "native GL symbol must be loaded");
    print("resolved native glGetStringi\n");
}
