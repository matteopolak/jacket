// File_Async: whole-file reads and writes, positioned reads and writes on an open file,
// completion order and user data, a full queue, missing files, and polling without waiting.
#import "Basic";
#import "File";
Async :: #import "File_Async";

ROOT :: ".scratch/file-async";

path :: (name: string) -> string {
    return tprint("%/%", ROOT, name);
}

main :: () {
    // The WASM target has no threads, which the queue's worker needs.
    #if OS == .WASM {
        print("ok\n");
        return;
    }

    delete_directory(ROOT);
    assert(make_directory_if_it_does_not_exist(ROOT, recursive = true));

    queue, created := Async.initialize_queue(int);
    assert(queue != null && created.code == .Success, "initialize_queue: %", created);

    // Nothing submitted: polling returns at once.
    _, nothing, polled := Async.wait_for_completion(queue, check_only = true);
    assert(polled.code == .DidNotWait && nothing.count == 0, "poll %", polled);

    {
        // A whole-file write, then a whole-file read of it.
        payload := "asynchronous ünïcode payload\n";
        result := Async.write_entire_file(queue, path("whole.txt"), cast([] u8) payload, 11);
        assert(result.code == .Success);
        tag, data, done := Async.wait_for_completion(queue);
        assert(tag == 11 && done.code == .Success, "write completion % %", tag, done);
        assert(cast(string) data == payload, "a write completion returns the caller's data");
        text, read_ok := read_entire_file(path("whole.txt"));
        assert(read_ok && text == payload, "written '%'", text);

        result = Async.read_entire_file(queue, path("whole.txt"), 12);
        assert(result.code == .Success);
        tag, data, done = Async.wait_for_completion(queue);
        assert(tag == 12 && done.code == .Success && cast(string) data == payload, "read completion % % '%'", tag, done, cast(string) data);
        free(data.data);
    }
    {
        // Several requests complete in submission order (one worker), each with its user data.
        names := string.["one.txt", "two.txt", "three.txt"];
        for names assert(write_entire_file(path(it), it));
        for names assert(Async.read_entire_file(queue, path(it), 100 + it_index).code == .Success);
        for names {
            tag, data, done := Async.wait_for_completion(queue);
            assert(done.code == .Success && tag == 100 + it_index && cast(string) data == it, "completion % '%'", tag, cast(string) data);
            free(data.data);
        }
        // An empty file reads as no bytes.
        assert(write_entire_file(path("empty.txt"), ""));
        assert(Async.read_entire_file(queue, path("empty.txt"), 7).code == .Success);
        tag, data, done := Async.wait_for_completion(queue);
        assert(tag == 7 && done.code == .Success && data.count == 0);
    }
    {
        // A missing file fails in the completion, not the submission.
        assert(Async.read_entire_file(queue, path("missing/none.txt"), 5).code == .Success);
        tag, data, done := Async.wait_for_completion(queue);
        assert(tag == 5 && done.code == .FileAccessFailed && data.count == 0, "missing file: %", done);
        assert(Async.write_entire_file(queue, path("missing/none.txt"), cast([] u8) "x", 6).code == .Success);
        tag, data, done = Async.wait_for_completion(queue);
        assert(tag == 6 && done.code == .FileAccessFailed, "write into a missing directory: %", done);
    }
    {
        // Positioned writes and reads on an open file, out of order.
        file, opened := Async.open_file(queue, path("random.bin"), keep_existing_content = false);
        assert(opened.code == .Success, "open_file %", opened);
        assert(Async.write_file(queue, file, 6, cast([] u8) "world", 1).code == .Success);
        assert(Async.write_file(queue, file, 0, cast([] u8) "hello,", 2).code == .Success);
        for 1..2 {
            tag, _, done := Async.wait_for_completion(queue);
            assert(tag == it && done.code == .Success, "positioned write %: %", tag, done);
        }
        buffer: [5] u8;
        assert(Async.read_file(queue, file, 6, buffer, 3).code == .Success);
        tag, data, done := Async.wait_for_completion(queue);
        assert(tag == 3 && done.code == .Success && data.data == buffer.data && cast(string) data == "world", "positioned read % '%'", done, cast(string) data);
        // Reading past the end comes back incomplete.
        assert(Async.read_file(queue, file, 9, buffer, 4).code == .Success);
        tag, data, done = Async.wait_for_completion(queue);
        assert(tag == 4 && done.code == .Incomplete, "read past the end: %", done);
        // A negative position is refused at submission.
        assert(Async.read_file(queue, file, -1, buffer, 0).code == .Catastrophic);
        Async.close_file(queue, file);
        // A closed file takes no more requests.
        assert(Async.read_file(queue, file, 0, buffer, 0).code == .FileAccessFailed);
        text, ok := read_entire_file(path("random.bin"));
        assert(ok && text == "hello,world", "random access file '%'", text);

        // Keeping the content of an existing file.
        kept, kept_ok := Async.open_file(queue, path("random.bin"));
        assert(kept_ok.code == .Success);
        assert(Async.write_file(queue, kept, 5, cast([] u8) " ", 8).code == .Success);
        tag, data, done = Async.wait_for_completion(queue);
        assert(tag == 8 && done.code == .Success);
        Async.close_file(queue, kept);
        text, ok = read_entire_file(path("random.bin"));
        assert(ok && text == "hello world", "kept and patched '%'", text);
    }
    {
        // More requests than the queue holds: the extra one is refused until completions drain.
        queue.capacity = 3;
        for 1..3 assert(Async.read_entire_file(queue, path("one.txt"), it).code == .Success);
        full := Async.read_entire_file(queue, path("one.txt"), 4);
        assert(full.code == .FullQueue, "full queue: %", full);
        for 1..3 {
            tag, data, done := Async.wait_for_completion(queue);
            assert(tag == it && done.code == .Success);
            free(data.data);
        }
        assert(Async.read_entire_file(queue, path("one.txt"), 9).code == .Success);
        tag, data, done := Async.wait_for_completion(queue);
        assert(tag == 9 && done.code == .Success);
        free(data.data);
    }
    {
        // Opening a file in a missing directory fails up front.
        _, failed := Async.open_file(queue, path("missing/x.bin"));
        assert(failed.code == .FileAccessFailed, "open in a missing directory: %", failed);
        // A request whose completion is never collected is released by destroy_queue.
        assert(Async.read_entire_file(queue, path("one.txt"), 1).code == .Success);
    }
    Async.destroy_queue(queue);
    // A null queue fails cleanly.
    none: *Async.Queue(int);
    assert(Async.read_entire_file(none, "x", 0).code == .Catastrophic);

    assert(delete_directory(ROOT));
    print("ok\n");
}
