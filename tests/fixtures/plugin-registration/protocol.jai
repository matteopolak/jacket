using #import "Basic";
C :: #import "Compiler";
P :: #import "Registration_Probe"(VALUE = 41);
#load "registration.jai";

main :: () {
    assert(!registration_ready());
    assert(!register_factory(0, 0, P.get_plugin));
    first := begin_registration(2);

    assert(!registration_ready());
    assert(!register_factory(first, -1, P.get_plugin));
    assert(!register_factory(first, 2, P.get_plugin));
    assert(!register_factory(first, 0, null));
    assert(!register_factory(first + 1, 0, P.get_plugin));
    assert(register_factory(first, 1, P.get_plugin));
    assert(!registration_ready());
    assert(!register_factory(first, 1, P.get_plugin));
    assert(register_factory(first, 0, P.get_plugin));
    assert(registration_ready() && registration_count == 2);
    assert(factory_slots[0] == P.get_plugin && factory_slots[1] == P.get_plugin);
    end_registration();
    assert(!registration_ready());
    assert(!register_factory(first, 0, P.get_plugin));
    second := begin_registration(1);

    assert(second != first);
    assert(!register_factory(first, 0, P.get_plugin));
    assert(!registration_ready());
    assert(register_factory(second, 0, P.get_plugin));
    assert(registration_ready());
    end_registration();
    empty := begin_registration(0);

    assert(empty != second && registration_ready());
    assert(!register_factory(empty, 0, P.get_plugin));
    end_registration();
    print("registration protocol ok\n");
}
