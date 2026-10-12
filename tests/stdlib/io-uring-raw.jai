// Extensions/Io_Uring through its raw ring: a NOP, a batch, a full submission queue, a file written
// and read back with IORING_OP_WRITE and IORING_OP_READ, the opcode probe, a timed wait, and a ring
// with 128-byte entries and 32-byte completions.
//
// Linux only, and only where the kernel lets the program use io_uring: on any other system, and on
// a Linux without io_uring (an old kernel, a container whose seccomp filter refuses the calls,
// kernel.io_uring_disabled), the test says why it skipped and exits 0.
#import "Basic";

#if OS == .LINUX {
    #import "File";
    Io_Uring :: #import "Extensions/Io_Uring";
}

main :: () {
    #if OS == .LINUX {
        run();
    } else {
        print("skipped: io_uring is a Linux kernel interface\n");
    }
}

#if OS == .LINUX {
    libc :: #system_library "libc";
    open :: (path: *u8, flags: s32, mode: u32) -> s32 #foreign libc;
    close :: (descriptor: s32) -> s32 #foreign libc;
    unlink :: (path: *u8) -> s32 #foreign libc;

    O_RDWR :: 2;
    O_CREAT :: 0x40;
    O_TRUNC :: 0x200;

    // errno values on every Linux CPU.
    EPERM :: 1;
    EACCES :: 13;
    ENOMEM :: 12;
    EINVAL :: 22;
    ENOSYS :: 38;
    ETIME :: 62;
    EAGAIN :: 11;

    ROOT :: ".scratch/io-uring-raw";

    // Did the kernel refuse to give us a ring at all, rather than something being wrong?
    unavailable :: (error: s32) -> bool {
        return error == -ENOSYS || error == -EPERM || error == -EACCES || error == -ENOMEM;
    }

    // Submits one entry and returns its completion's result.
    complete :: (ring: *Io_Uring.Ring, user_data: u64) -> s32 {
        assert(Io_Uring.submit_and_wait(ring, 1) >= 1, "submit_and_wait");

        cqe: *Io_Uring.io_uring_cqe;

        assert(Io_Uring.wait_cqe(ring, *cqe) == 0, "wait_cqe");
        assert(cqe.user_data == user_data, "the completion carries user_data %, got %", user_data, cqe.user_data);

        result := cqe.res;

        Io_Uring.cqe_seen(ring, cqe);

        return result;
    }

    run :: () {
        ring: Io_Uring.Ring;
        made := Io_Uring.ring_init(*ring, 4);

        if unavailable(made) {
            print("skipped: io_uring is not available here (ring_init returned -%)\n", -made);

            return;
        }

        assert(made == 0, "ring_init failed with -%", -made);
        defer Io_Uring.ring_deinit(*ring);

        assert(ring.sq_entries == 4 && ring.cq_entries >= 4, "ring sizes % %", ring.sq_entries, ring.cq_entries);

        // Nothing submitted, nothing complete.
        cqe: *Io_Uring.io_uring_cqe;

        assert(Io_Uring.peek_cqe(*ring, *cqe) == -EAGAIN, "an idle ring has no completion");

        // One NOP.
        sqe := Io_Uring.get_sqe(*ring);

        assert(sqe != null);
        Io_Uring.prep_nop(sqe);
        sqe.user_data = 0x1234_5678_9abc;
        assert(complete(*ring, 0x1234_5678_9abc) == 0, "a NOP completes with 0");

        // The probe: NOP exists in every kernel that has the probe, and so do READ and WRITE.
        probe: Io_Uring.Probe;
        probed := Io_Uring.probe_ops(*ring, *probe);

        assert(probed == 0 || probed == -EINVAL, "probe_ops returned %", probed);
        have_read_write := false;

        if probed == 0 {
            assert(Io_Uring.supports(*probe, .NOP), "the probe lists NOP");
            assert(probe.last_op >= cast(u8) Io_Uring.io_uring_op.WRITE, "last_op %", probe.last_op);
            have_read_write = Io_Uring.supports(*probe, .READ) && Io_Uring.supports(*probe, .WRITE);
            assert(have_read_write, "a kernel with the probe has READ and WRITE");
            assert(!probe.supported[255], "opcode 255 does not exist");
            print("probe: last_op %, features %\n", probe.last_op, ring.features);
        }

        // A batch fills the submission queue; the next get_sqe says it is full until submit.
        for 1..4 {
            batch := Io_Uring.get_sqe(*ring);
            assert(batch != null, "entry % of 4", it);
            Io_Uring.prep_nop(batch);
            batch.user_data = cast(u64) it;
        }

        assert(Io_Uring.get_sqe(*ring) == null, "a full submission queue hands out no entry");
        assert(Io_Uring.sq_pending(*ring) == 4);
        assert(Io_Uring.submit_and_wait(*ring, 4) == 4, "four entries submitted");
        assert(Io_Uring.sq_pending(*ring) == 0);
        assert(Io_Uring.cq_ready(*ring) == 4, "four completions ready, got %", Io_Uring.cq_ready(*ring));

        seen_mask: u64;

        for 1..4 {
            assert(Io_Uring.wait_cqe(*ring, *cqe) == 0);
            assert(cqe.res == 0 && cqe.user_data >= 1 && cqe.user_data <= 4);
            seen_mask |= cast(u64) 1 << cqe.user_data;
            Io_Uring.cqe_seen(*ring, cqe);
        }

        assert(seen_mask == 0b11110, "every completion arrives once: %", seen_mask);
        // Submitting made room. An entry nobody filled in is a NOP, since opcode 0 is NOP.
        spare := Io_Uring.get_sqe(*ring);

        assert(spare != null, "submitting made room");
        spare.user_data = 9;
        assert(complete(*ring, 9) == 0);

        // A timed wait on an idle ring times out.
        if ring.features & Io_Uring.IORING_FEAT_EXT_ARG {
            timed := Io_Uring.wait_cqe_timeout(*ring, *cqe, 2_000_000);
            assert(timed == -ETIME, "wait_cqe_timeout returned %", timed);
        }

        if have_read_write {
            check_file(*ring);
        }

        check_big_entries();
        print("io_uring ok\n");
    }

    check_file :: (ring: *Io_Uring.Ring) {
        make_directory_if_it_does_not_exist(ROOT, recursive = true);
        path := tprint("%/data.bin\0", ROOT);
        fd := open(path.data, O_RDWR | O_CREAT | O_TRUNC, 420);

        assert(fd >= 0, "open %", path);

        defer {
            close(fd);
            unlink(path.data);
        }

        message := "hello from io_uring, written with IORING_OP_WRITE";

        write_sqe := Io_Uring.get_sqe(ring);

        Io_Uring.prep_rw(write_sqe, .WRITE, fd, message.data, cast(u32) message.count, 0);
        write_sqe.user_data = 100;
        written := complete(ring, 100);

        assert(written == message.count, "wrote % of % bytes", written, message.count);

        // Read it back at an offset, into a buffer that starts out different.
        buffer: [64] u8;
        for *buffer it.* = #char "?";

        skip := 6;

        read_sqe := Io_Uring.get_sqe(ring);

        Io_Uring.prep_rw(read_sqe, .READ, fd, buffer.data, cast(u32) buffer.count, cast(u64) skip);
        read_sqe.user_data = 101;
        got := complete(ring, 101);

        assert(got == message.count - skip, "read % bytes, wanted %", got, message.count - skip);

        text: string = .{ count = got, data = buffer.data };

        assert(text == "from io_uring, written with IORING_OP_WRITE", "read back '%'", text);
        assert(buffer[got] == #char "?", "the read stopped at the end of the file");

        // A read at the end of the file completes with 0; a bad descriptor with a negative errno.
        eof := Io_Uring.get_sqe(ring);

        Io_Uring.prep_rw(eof, .READ, fd, buffer.data, cast(u32) buffer.count, cast(u64) message.count);
        eof.user_data = 102;
        assert(complete(ring, 102) == 0, "reading at the end of the file gives 0");

        bad := Io_Uring.get_sqe(ring);

        Io_Uring.prep_rw(bad, .READ, -1, buffer.data, cast(u32) buffer.count, 0);
        bad.user_data = 103;
        failed := complete(ring, 103);

        assert(failed < 0, "reading a bad descriptor fails with a negative errno, got %", failed);
    }

    check_big_entries :: () {
        ring: Io_Uring.Ring;
        made := Io_Uring.ring_init(*ring, 2, Io_Uring.IORING_SETUP_SQE128 | Io_Uring.IORING_SETUP_CQE32);

        if made < 0 {
            print("skipped the 128-byte entries: ring_init returned -%\n", -made);

            return;
        }

        defer Io_Uring.ring_deinit(*ring);

        assert(ring.sqe_bytes == 128 && ring.cqe_bytes == 32);

        // Two entries fill the ring: the second must not overlap the first.
        first := cast(*Io_Uring.io_uring_sqe128) Io_Uring.get_sqe(*ring);
        second := cast(*Io_Uring.io_uring_sqe128) Io_Uring.get_sqe(*ring);

        assert(cast(u64) second - cast(u64) first == 128, "entries are 128 bytes apart");
        Io_Uring.prep_nop(*first.sqe);
        first.user_data = 7;
        Io_Uring.prep_nop(*second.sqe);
        second.user_data = 8;
        assert(Io_Uring.submit_and_wait(*ring, 2) == 2);

        total: u64;

        for 1..2 {
            cqe: *Io_Uring.io_uring_cqe;

            assert(Io_Uring.wait_cqe(*ring, *cqe) == 0);

            big := cast(*Io_Uring.io_uring_cqe32) cqe;

            assert(big.res == 0 && big.extra[0] == 0 && big.extra[1] == 0);
            total += big.user_data;
            Io_Uring.cqe_seen(*ring, cqe);
        }

        assert(total == 15, "both completions arrived: %", total);
    }
}
