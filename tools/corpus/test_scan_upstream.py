import sys
import tempfile
import unittest
from pathlib import Path

import scan_upstream as scan


def hits(source: str, name: str = "main.jai"):
    return [(f.rule, f.risk, f.context) for f in scan.scan_text(name, source, name)]


class RuntimeVersusBuild(unittest.TestCase):
    def test_runtime_process_spawn_is_medium(self):
        self.assertIn(("process-spawn", "medium", "runtime"), hits('main :: () { run_command("ls"); }'))

    def test_the_same_call_in_run_is_high(self):
        self.assertIn(("process-spawn", "high", "build"), hits('x :: #run run_command("ls");\nmain :: () {}'))

    def test_a_procedure_called_from_run_is_build_code(self):
        source = 'helper :: () {\n    run_command("sh");\n}\nmain :: () {}\n#run helper();\n'
        self.assertIn(("process-spawn", "high", "build"), hits(source))

    def test_build_program_names_make_the_whole_file_build_code(self):
        self.assertIn(("process-spawn", "high", "build"), hits('build :: () { run_command("cc"); }', "build.jai"))
        self.assertIn(("process-spawn", "high", "build"), hits('f :: () { run_command("cc"); }\n', "first.jai"))

    def test_comments_are_ignored(self):
        self.assertEqual(hits('// run_command("rm -rf /")\n/* system("x") */ main :: () {}'), [])

    def test_strings_do_not_count_as_calls(self):
        self.assertNotIn("process-spawn", [h[0] for h in hits('s :: "run_command(x)";')])


class Rules(unittest.TestCase):
    def test_network(self):
        self.assertIn("network-import", [h[0] for h in hits('#import "Socket";')])
        self.assertIn(("network", "high", "build"), hits('#run connect(1, 2);\n'))

    def test_url_and_ip_are_high_only_at_build_time(self):
        self.assertIn(("url", "low", "runtime"), hits('u :: "https://example.com/a";'))
        self.assertIn(("url", "high", "build"), hits('#run print("https://example.com/a");'))
        self.assertIn(("ip-literal", "medium", "runtime"), hits('ip :: "8.8.8.8";'))
        self.assertNotIn("ip-literal", [h[0] for h in hits('ip :: "127.0.0.1";')])

    def test_deletes_and_outside_paths(self):
        self.assertIn(("file-delete", "medium", "build"), hits('#run file_delete("out.tmp");'))
        self.assertIn(("file-delete", "high", "build"), hits('#run file_delete("/etc/hosts");'))
        self.assertIn(("file-delete", "high", "build"), hits('#run file_delete(path);'))
        self.assertIn(("outside-path", "high", "build"), hits('#run write_entire_file("../x", "y");'))
        self.assertIn(("outside-path", "high", "build"), hits('#run write_entire_file("~/x", "y");'))

    def test_imports_may_climb(self):
        self.assertNotIn("outside-path", [h[0] for h in hits('#import,file "../module.jai";', "build.jai")])

    def test_credentials_are_high_anywhere(self):
        self.assertIn(("credential-path", "high", "runtime"), hits('p :: "/home/u/.ssh/id_rsa";'))

    def test_environment_is_high_at_build_time_only(self):
        self.assertIn(("environment", "high", "build"), hits('#run getenv("HOME");'))
        self.assertIn(("environment", "low", "runtime"), hits('x := getenv("DISPLAY");'))

    def test_foreign_libraries(self):
        self.assertNotIn("foreign-library", [h[0] for h in hits('gl :: #library "opengl32";')])
        self.assertIn(("foreign-library", "medium", "runtime"), hits('x :: #library "zzqy_helper";'))
        self.assertIn(("foreign-network-library", "medium", "runtime"), hits('w :: #system_library "ws2_32";'))
        self.assertIn(("foreign-library-path", "medium", "runtime"), hits('x :: #library "../evil/lib";'))
        self.assertEqual([h for h in hits('print("#library \\"%\\"");') if h[0].startswith("foreign")], [])

    def test_asm(self):
        self.assertIn(("asm", "medium", "runtime"), hits("f :: () { #asm { nop; } }"))

    def test_blobs(self):
        blob = "A" * 300
        self.assertIn(("blob-base64", "medium", "runtime"), hits(f's :: "{blob}";'))
        self.assertIn(("blob-base64", "high", "runtime"), hits('s :: "' + "TVqQAAMAAAAEAAAA" * 20 + '";'))
        array = "u8.[" + ", ".join(["1"] * 300) + "]"
        self.assertIn(("byte-array", "medium", "runtime"), hits(f"d :: {array};"))
        header = "u8.[0x7f, 0x45, 0x4c, 0x46, " + ", ".join(["1"] * 300) + "]"
        self.assertIn(("byte-array", "high", "runtime"), hits(f"d :: {header};"))

    def test_obfuscated_identifiers(self):
        self.assertIn("obfuscated-identifier", [h[0] for h in hits("_0x1f2e3d := 1;")])
        self.assertIn("obfuscated-identifier", [h[0] for h in hits("lIlIlIlIlIlI := 1;")])
        self.assertNotIn("obfuscated-identifier", [h[0] for h in hits("PFNGLPATHGLYPHSNVPROC :: 1; vkCreateGraphicsPipelines :: 2;")])


class Directories(unittest.TestCase):
    def test_binary_files_are_high_and_only_pinned_paths_are_read(self):
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            (root / "a.jai").write_text('main :: () {}\n')
            (root / "lib.dll").write_bytes(b"MZ\x90\x00")
            (root / "disguised.txt").write_bytes(b"\x7fELF\x02")
            (root / "stray.jai").write_text('#run system("x");')
            found = scan.scan_directory(root)
            self.assertEqual(sorted(f.path for f in found if f.rule == "binary-file"), ["disguised.txt", "lib.dll"])
            self.assertEqual(scan.risk_of(found), "high")
            pinned = scan.scan_directory(root, {"a.jai"})
            self.assertEqual(pinned, [])

    def test_render_lists_every_high_line(self):
        text = scan.render("o--r", scan.scan_text("first.jai", 'f :: () { run_command("x"); }'))
        self.assertIn("o--r: HIGH", text)
        self.assertIn("first.jai:1:", text)


if __name__ == "__main__":
    unittest.main()
