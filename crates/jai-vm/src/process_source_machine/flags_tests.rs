use super::*;

fn call(fixture: &mut Fixture, fd: i32, command: i32, flags: Option<i32>) -> i128 {
    let mut args = vec![s32(fd), s32(command)];
    if let Some(flags) = flags {
        args.push(s32(flags));
    }
    scalar(fixture.call(ProcessAbiOperation::Fcntl, &args).unwrap())
}

#[test]
fn source_nested_flag_getters_setters_preserve_actual_access_and_target_bits() {
    for os in [OperatingSystem::MacOS, OperatingSystem::Linux] {
        let nonblock = if os == OperatingSystem::MacOS {
            4
        } else {
            0x800
        };
        let mut fixture = Fixture::new_for(os);
        let [read, write] = fixture.pipe();
        assert_eq!(call(&mut fixture, read, 3, None), 0);
        assert_eq!(call(&mut fixture, write, 3, None), 1);
        let current = call(&mut fixture, write, 3, None) as i32;
        assert_eq!(call(&mut fixture, write, 4, Some(current | nonblock)), 0);
        assert_eq!(call(&mut fixture, write, 3, None), i128::from(1 | nonblock));
        assert_eq!(call(&mut fixture, write, 4, Some(1)), 0);
        assert_eq!(call(&mut fixture, write, 3, None), 1);
        let current = call(&mut fixture, read, 1, None) as i32;
        assert_eq!(call(&mut fixture, read, 2, Some(current | 1)), 0);
        assert_eq!(call(&mut fixture, read, 1, None), 1);
        assert_eq!(call(&mut fixture, read, 2, Some(0)), 0);
        assert_eq!(call(&mut fixture, read, 1, None), 0);
        let pair = fixture.world.socketpair(fixture.branch.current()).unwrap();
        assert_eq!(call(&mut fixture, pair[0].number(), 3, None), 2);
        fixture.world.shutdown_write(pair[0]).unwrap();
        assert_eq!(call(&mut fixture, pair[0].number(), 3, None), 2);
    }
}

#[test]
fn descriptor_flags_are_private_slots_but_status_is_shared_across_dup_fork_and_rights() {
    let mut fixture = Fixture::new();
    let root = fixture.branch.current();
    let [read, _] = fixture.pipe();
    let fd = fixture.world.descriptor(root, read).unwrap();
    let duplicate = fixture.world.dup2(fd, 10).unwrap();
    assert_eq!(call(&mut fixture, read, 2, Some(1)), 0);
    let child = fixture.world.fork(root).unwrap().child;
    let child_fd = fixture.world.descriptor(child, read).unwrap();
    assert!(fixture.world.close_on_exec(fd).unwrap());
    assert!(!fixture.world.close_on_exec(duplicate).unwrap());
    assert!(fixture.world.close_on_exec(child_fd).unwrap());
    fixture.world.set_close_on_exec(child_fd, false).unwrap();
    assert!(!fixture.world.close_on_exec(child_fd).unwrap());
    assert!(fixture.world.close_on_exec(fd).unwrap());
    assert_eq!(call(&mut fixture, read, 4, Some(4)), 0);
    assert!(fixture.world.nonblocking(duplicate).unwrap());
    assert!(fixture.world.nonblocking(child_fd).unwrap());
    let sockets = fixture.world.socketpair(root).unwrap();
    fixture.world.send_rights(sockets[0], b"x", &[fd]).unwrap();
    let ProcessIo::Ready(message) = fixture.world.recv_rights(sockets[1], 1, 1).unwrap() else {
        panic!("queued rights message must be ready")
    };
    let received = message.descriptors[0];
    assert!(!fixture.world.close_on_exec(received).unwrap());
    assert!(fixture.world.nonblocking(received).unwrap());
    assert_eq!(call(&mut fixture, received.number(), 4, Some(0)), 0);
    assert!(!fixture.world.nonblocking(fd).unwrap());
    assert!(!fixture.world.nonblocking(child_fd).unwrap());
    assert!(fixture.world.close_on_exec(fd).unwrap());
    assert!(!fixture.world.close_on_exec(received).unwrap());
}

#[test]
fn fcntl_bad_descriptor_updates_real_errno_even_after_nested_getter_failure() {
    let mut fixture = Fixture::new();
    assert_eq!(call(&mut fixture, 99, 1, None), -1);
    assert_eq!(fixture.errno().1, 9);
    // Actual source ORs the failed getter's -1 into the setter flags.
    assert_eq!(call(&mut fixture, 99, 2, Some(-1)), -1);
    assert_eq!(fixture.errno().1, 9);
    let [read, _] = fixture.pipe();
    assert_eq!(call(&mut fixture, read, 1, None), 0);
    assert_eq!(fixture.errno().1, 9);
    fixture
        .world
        .close(
            fixture
                .world
                .descriptor(fixture.branch.current(), read)
                .unwrap(),
        )
        .unwrap();
    assert_eq!(call(&mut fixture, read, 3, None), -1);
}

#[test]
fn fcntl_rejects_unknown_flags_command_arity_and_nonpromoted_c_tail_without_mutation() {
    let mut fixture = Fixture::new();
    let [read, write] = fixture.pipe();
    let fd = fixture
        .world
        .descriptor(fixture.branch.current(), write)
        .unwrap();
    for args in [
        vec![s32(write), s32(2), s32(2)],
        vec![s32(write), s32(4), s32(0)],
        vec![s32(write), s32(4), s32(1 | 0x800)],
        vec![s32(write), s32(99)],
        vec![s32(write), s32(1), s32(0)],
        vec![s32(write), s32(2)],
        vec![s32(write), s32(2), int(IntegerType::S64, 1)],
        vec![s32(write), s32(2), int(IntegerType::U32, 1)],
        vec![int(IntegerType::S64, i128::from(write)), s32(1)],
    ] {
        assert!(fixture.call(ProcessAbiOperation::Fcntl, &args).is_err());
        assert!(!fixture.world.close_on_exec(fd).unwrap());
        assert!(!fixture.world.nonblocking(fd).unwrap());
        assert!(fixture.branch.errno.is_empty());
    }
    let pointer = fixture.pair_buffer();
    let address = Value::AddressInteger(Number::address(
        Integer::checked(IntegerType::S32, 1).unwrap(),
        crate::AddressProvenance::Pointer(pointer.clone()),
    ));
    assert!(
        fixture
            .call(ProcessAbiOperation::Fcntl, &[s32(read), s32(2), address])
            .is_err()
    );
    assert!(
        fixture
            .call(
                ProcessAbiOperation::Fcntl,
                &[s32(read), s32(2), Value::Pointer(pointer)]
            )
            .is_err()
    );
}

#[test]
fn fcntl_fuel_rejection_precedes_flags_or_errno_mutation_and_proof_revalidates_target() {
    let mut fixture = Fixture::new();
    let [read, _] = fixture.pipe();
    let proof = fixture.proof(ProcessAbiOperation::Fcntl);
    for args in [vec![s32(read), s32(2), s32(1)], vec![s32(99), s32(1)]] {
        assert!(matches!(
            proof.invoke_process(
                &args,
                &mut fixture.memory,
                &fixture.types,
                &fixture.target,
                &mut fixture.branch,
                &mut fixture.world,
                &mut |_| Err(Error::Limit(LimitKind::Fuel))
            ),
            Err(Error::Limit(LimitKind::Fuel))
        ));
    }
    let fd = fixture
        .world
        .descriptor(fixture.branch.current(), read)
        .unwrap();
    assert!(!fixture.world.close_on_exec(fd).unwrap());
    assert!(fixture.branch.errno.is_empty());
    let mut wrong = fixture.target.clone();
    wrong.operating_system = OperatingSystem::Linux;
    assert!(
        proof
            .validate_process_arguments(&[s32(read), s32(1)], &fixture.types, &wrong)
            .is_err()
    );
    assert!(
        proof
            .validate_process_arguments(
                &[s32(read), s32(2), s32(1)],
                &fixture.types,
                &fixture.target
            )
            .is_ok()
    );
}

#[test]
fn fcntl_setters_admit_both_world_copies_before_cloning_or_mutating_flags() {
    let mut fixture = Fixture::new();
    let [read, _] = fixture.pipe();
    let world_cells = usize::try_from(fixture.world.work_cost().unwrap()).unwrap();
    let limit = world_cells + world_cells / 2 + 3;
    fixture.memory = Memory::new(Limits {
        value_cells: limit,
        ..Default::default()
    });
    // One world and the arguments fit, while the candidate copy does not.
    assert!(world_cells + 3 <= limit);
    assert!(world_cells * 2 + 3 > limit);
    for args in [
        vec![s32(read), s32(2), s32(1)],
        vec![s32(read), s32(4), s32(4)],
    ] {
        assert!(matches!(
            fixture.call(ProcessAbiOperation::Fcntl, &args),
            Err(Error::Limit(LimitKind::ValueCells))
        ));
    }
    let fd = fixture
        .world
        .descriptor(fixture.branch.current(), read)
        .unwrap();
    assert!(!fixture.world.close_on_exec(fd).unwrap());
    assert!(!fixture.world.nonblocking(fd).unwrap());
    assert!(fixture.branch.errno.is_empty());
    assert_eq!(call(&mut fixture, read, 1, None), 0);
    assert_eq!(call(&mut fixture, read, 3, None), 0);
}
