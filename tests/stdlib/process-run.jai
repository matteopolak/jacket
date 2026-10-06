// Process with real children: captured stdout and stderr, exit codes, a missing program, a
// working directory, writing to a child's stdin, large output, timeouts, kills, writing to a
// child that has exited, and the command-line splitting and quoting helpers.
#import "Basic";
#import "File";
#import "Process";
#import "String";

// The shell for one command line: `/bin/sh -c` on POSIX, `cmd /c` on Windows.
#if OS == .WINDOWS {
    SHELL :: string.["cmd", "/c"];
} else {
    SHELL :: string.["/bin/sh", "-c"];
}

shell :: (command: string, working_directory := "", timeout_ms := -1) -> Process_Result, string, string, bool {
    result, out, err, timed_out := run_command(SHELL[0], SHELL[1], command, working_directory = working_directory, capture_and_return_output = true, timeout_ms = timeout_ms);
    // cmd ends lines with CR LF.
    return result, replace(out, "\r", ""), replace(err, "\r", ""), timed_out;
}

main :: () {
    helpers();
    // The WASM target starts no processes.
    #if OS == .WASM {
        print("ok\n");
        return;
    }

    {
        result, out, err, timed_out := shell("echo hello");
        assert(result.type == .EXITED && result.exit_code == 0 && !timed_out, "echo: %", result);
        assert(out == "hello\n" && err == "", "echo out '%' err '%'", out, err);
    }
    {
        result, out, err := shell("echo to-stderr 1>&2");
        assert(result.type == .EXITED && result.exit_code == 0);
        assert(out == "" && trim(err) == "to-stderr", "stderr out '%' err '%'", out, err);
    }
    {
        result := shell("exit 3");
        assert(result.type == .EXITED && result.exit_code == 3, "exit 3: %", result);
        // Without capturing, the result still comes back.
        plain := run_command(SHELL[0], SHELL[1], "exit 5");
        assert(plain.type == .EXITED && plain.exit_code == 5, "uncaptured exit 5: %", plain);
    }
    {
        // A program that does not exist fails to launch, cleanly.
        result, out := run_command("jaic-no-such-program-for-this-test", "x", capture_and_return_output = true);
        assert(result.type == .FAILED_TO_LAUNCH && out == "", "missing program: %", result);
        process: Process;
        assert(!create_process(*process, "jaic-no-such-program-for-this-test"));
        assert(!process.launched);
        // No arguments at all.
        empty: [] string;
        assert(!create_process(*process, ..empty));
        assert(run_command(..empty).type == .FAILED_TO_LAUNCH);
    }
    {
        // The working directory applies to the child only.
        scratch := ".scratch/process-run";
        delete_directory(scratch);
        assert(make_directory_if_it_does_not_exist(scratch, recursive = true));
        assert(write_entire_file(tprint("%/marker.txt", scratch), "marker contents"));
        #if OS == .WINDOWS {
            listing :: "type marker.txt";
        } else {
            listing :: "cat marker.txt";
        }
        result, out := shell(listing, working_directory = scratch);
        assert(result.type == .EXITED && result.exit_code == 0 && out == "marker contents", "in the working directory: % '%'", result, out);
        result = shell(listing);
        assert(result.type == .EXITED && result.exit_code != 0, "the parent's directory is unchanged");
        // A missing working directory fails to launch.
        missing := run_command(SHELL[0], SHELL[1], "exit 0", working_directory = "no/such/dir");
        assert(missing.type == .FAILED_TO_LAUNCH, "missing working directory: %", missing);
        assert(delete_directory(scratch));
    }
    {
        // Output larger than a pipe buffer on both streams does not deadlock.
        #if OS == .WINDOWS {
            noisy :: "for /L %i in (1,1,6000) do @(echo out-line-%i& echo err-line-%i 1>&2)";
        } else {
            noisy :: "i=1; while [ $i -le 6000 ]; do echo out-line-$i; echo err-line-$i >&2; i=$((i+1)); done";
        }
        result, out, err := shell(noisy);
        assert(result.type == .EXITED && result.exit_code == 0, "noisy: %", result);
        out_lines := split(trim_right(out, "\n"), "\n");
        err_lines := split(trim_right(err, "\n"), "\n");
        assert(out_lines.count == 6000 && err_lines.count == 6000, "% and % lines", out_lines.count, err_lines.count);
        assert(trim(out_lines[5999]) == "out-line-6000" && trim(err_lines[0]) == "err-line-1", "last '%' first '%'", out_lines[5999], err_lines[0]);
    }
    {
        // Writing to a child's stdin, then closing it so the child sees the end.
        process: Process;
        assert(create_process(*process, "sort", capture_and_return_output = true));
        ok, written := write_to_process(*process, cast([] u8) "pear\napple\nfig\n");
        assert(ok && written == 15, "wrote %", written);
        deinit(*process.input);
        collected: String_Builder;
        out_buffer: [4096] u8;
        err_buffer: [4096] u8;
        while !process.output.eof || !process.error.eof {
            read_ok, out_count := read_from_process(*process, out_buffer, err_buffer, 5000);
            assert(read_ok);
            append(*collected, cast(string) array_view(out_buffer, 0, out_count));
        }
        done, result := get_process_result(*process);
        assert(done && result.type == .EXITED && result.exit_code == 0, "sort: %", result);
        sorted := replace(builder_to_string(*collected), "\r", "");
        assert(sorted == "apple\nfig\npear\n", "sorted '%'", sorted);
        deinit(*process);
        assert(!process.launched);
    }
    {
        // run_command gives a captured child no input: one that reads stdin sees its end at
        // once instead of waiting forever.
        result, out, _, timed_out := run_command("sort", capture_and_return_output = true, timeout_ms = 5000);
        assert(!timed_out && result.type == .EXITED && result.exit_code == 0 && out == "", "sort without input: % % '%'", timed_out, result, out);
    }
    {
        // A child that has exited: writing to it fails instead of killing us with SIGPIPE.
        process: Process;
        assert(create_process(*process, SHELL[0], SHELL[1], "exit 0", capture_and_return_output = true));
        done, result := get_process_result(*process);
        assert(done && result.type == .EXITED);
        // The result is remembered.
        done, result = get_process_result(*process, 0);
        assert(done && result.type == .EXITED && result.exit_code == 0);
        failed := false;
        for 1..20 {
            ok := write_to_process(*process, cast([] u8) "nobody is reading\n");
            if !ok {
                failed = true;
                break;
            }
        }
        assert(failed, "writes to an exited child fail");
        deinit(*process);
    }
    {
        // A timeout kills a child that runs too long, promptly.
        #if OS == .WINDOWS {
            slow :: string.["ping", "-n", "30", "127.0.0.1"];
        } else {
            slow :: string.["sleep", "30"];
        }
        started := current_time_monotonic();
        result, _, _, timed_out := run_command(..slow, capture_and_return_output = true, timeout_ms = 300);
        elapsed := to_milliseconds(current_time_monotonic() - started);
        assert(timed_out && elapsed < 10000, "timed out % after % ms", timed_out, elapsed);
        assert(result.type == .EXITED || result.type == .SIGNALED, "killed: %", result);
        result, _, _, timed_out = run_command(..slow, timeout_ms = 300);
        assert(timed_out, "uncaptured timeout");

        // kill_process on a running child.
        process: Process;
        assert(create_process(*process, ..slow));
        done, still := get_process_result(*process, 0);
        assert(done && still.type == .STILL_RUNNING, "running: %", still);
        assert(kill_process(*process));
        killed: Process_Result;
        done, killed = get_process_result(*process);
        #if OS == .WINDOWS {
            assert(done && killed.type == .EXITED && killed.exit_code == 137, "killed: %", killed);
        } else {
            assert(done && killed.type == .SIGNALED && killed.signal == 9, "killed: %", killed);
        }
        deinit(*process);
        // A process that never started.
        never: Process;
        unstarted: Process_Result;
        done, unstarted = get_process_result(*never);
        assert(done && unstarted.type == .UNSTARTED);
        assert(!kill_process(*never));
    }
    {
        // run_command_line splits the command first.
        #if OS == .WINDOWS {
            code, out := run_command_line("cmd /c \"echo split line& exit 7\"", capture_and_return_output = true);
        } else {
            code, out := run_command_line("/bin/sh -c 'echo split line; exit 7'", capture_and_return_output = true);
        }
        assert(code == 7 && trim(out) == "split line", "run_command_line % '%'", code, out);
        result := run_command_arguments(SHELL[0], SHELL[1], "exit 2");
        assert(result.type == .EXITED && result.exit_code == 2);
        #if OS != .WINDOWS {
            signaled := run_command_line("/bin/sh -c 'kill -9 $$'");
            assert(signaled == 128 + 9, "signaled code %", signaled);
        }
    }
    {
        // Arguments reach the child exactly, spaces, quotes and empty ones included.
        #if OS == .WINDOWS {
            result, out := run_command("cmd", "/c", "echo", "a b", "plain", capture_and_return_output = true);
            assert(result.exit_code == 0 && trim(out) == "\"a b\" plain", "echoed '%'", out);
        } else {
            result, out := run_command("/bin/sh", "-c", "printf '%s|' \"$@\"", "sh", "a b", "q\"uote", "", "back\\slash", capture_and_return_output = true);
            assert(result.exit_code == 0 && out == "a b|q\"uote||back\\slash|", "arguments '%'", out);
        }
    }
    // Every child above was reaped and released.
    assert(thread_is_done());
    shutdown();
    print("ok\n");
}

helpers :: () {
    words := break_command_into_strings("  plain \"double quoted\" 'single quoted'\ttab\\ escaped mixed\"quo\"ted \"\" end  ");
    expected :: string.["plain", "double quoted", "single quoted", "tab escaped", "mixedquoted", "", "end"];
    assert(words.count == expected.count, "% words", words.count);
    for expected assert(words[it_index] == it, "word % is '%', expected '%'", it_index, words[it_index], it);
    assert(break_command_into_strings("").count == 0);
    assert(break_command_into_strings("   \t\n").count == 0);
    escaped := break_command_into_strings("a\\\"b 'c\\d'");
    assert(escaped.count == 2 && escaped[0] == "a\"b" && escaped[1] == "c\\d", "escapes '%' '%'", escaped[0], escaped[1]);

    // Windows command-line quoting: quotes only when needed, backslashes doubled before quotes.
    assert(get_quoted_command_string(.["prog", "simple", "C:\\dir\\file"]) == "prog simple C:\\dir\\file");
    assert(get_quoted_command_string(.["a b", ""]) == "\"a b\" \"\"");
    quoted := get_quoted_command_string(.["say \"hi\"", "end\\ with\\"]);
    assert(quoted == "\"say \\\"hi\\\"\" \"end\\ with\\\\\"", "quoted '%'", quoted);
    assert(get_quoted_command_string(.[]) == "");
}
