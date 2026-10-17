// Independent registrar primitive for the library owner; no servicing guarantee.
Factory :: #type () -> *C.Metaprogram_Plugin;
factory_slots: [..] Factory;
registration_epoch: int;
registration_count: int;
registration_active: bool;

begin_registration :: (count: int) -> int {
    assert(!registration_active && count >= 0);
    registration_epoch += 1;
    registration_count = 0;
    array_resize(*factory_slots, count);
    registration_active = true;

    return registration_epoch;
}

register_factory :: (epoch: int, index: int, value: Factory) -> bool {
    if !registration_active || epoch != registration_epoch return false;
    if index < 0 || index >= factory_slots.count || value == null return false;
    if factory_slots[index] != null return false;

    factory_slots[index] = value;
    registration_count += 1;

    return true;
}

registration_ready :: () -> bool {
    return registration_active && registration_count == factory_slots.count;
}

end_registration :: () {
    assert(registration_ready());
    array_reset(*factory_slots);
    registration_active = false;
}
