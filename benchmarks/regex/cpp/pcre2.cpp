// PCRE2 harness with the JIT. UTF + UCP, so \w, \b and \p{..} are Unicode-aware.
#define PCRE2_CODE_UNIT_WIDTH 8
#include <pcre2.h>

#include "common.h"

int main(int argc, char** argv) {
    Harness h = parse_args(argc, argv);
    const std::string& text = h.corpus;
    const uint32_t flags = PCRE2_UTF | PCRE2_UCP;
    pcre2_jit_stack* stack = pcre2_jit_stack_create(32 * 1024, 16 * 1024 * 1024, nullptr);
    pcre2_match_context* mctx = pcre2_match_context_create(nullptr);
    pcre2_jit_stack_assign(mctx, nullptr, stack);
    for (const Case& c : h.cases) {
        auto compile_once = [&](int* err, PCRE2_SIZE* eoff) {
            pcre2_code* code = pcre2_compile((PCRE2_SPTR)c.pattern.c_str(), c.pattern.size(), flags, err, eoff, nullptr);
            if (code && pcre2_jit_compile(code, PCRE2_JIT_COMPLETE) != 0) {
                pcre2_code_free(code);
                return (pcre2_code*)nullptr;
            }
            return code;
        };
        int err = 0;
        PCRE2_SIZE eoff = 0;
        double compile = 1e30;
        for (int i = 0; i < 20; i++) {
            double t = now_seconds();
            pcre2_code* code = compile_once(&err, &eoff);
            compile = std::min(compile, now_seconds() - t);
            if (code) pcre2_code_free(code);
        }
        pcre2_code* code = compile_once(&err, &eoff);
        if (!code) {
            PCRE2_UCHAR msg[256];
            pcre2_get_error_message(err, msg, sizeof msg);
            report_na("pcre2-jit", c.id, std::string("compile error: ") + (const char*)msg);
            continue;
        }
        pcre2_match_data* md = pcre2_match_data_create_from_pattern(code, nullptr);
        bool caps = c.id == "caps";
        Outcome out;
        double best = best_of(h.runs, out, [&]() {
            Outcome o;
            PCRE2_SIZE pos = 0;
            while (pos <= text.size()) {
                int rc = pcre2_jit_match(code, (PCRE2_SPTR)text.data(), text.size(), pos, 0, md, mctx);
                if (rc < 0) break;
                PCRE2_SIZE* ov = pcre2_get_ovector_pointer(md);
                o.count++;
                if (caps)
                    for (int g = 1; g <= 3; g++) o.check += to_u64(text.data() + ov[2 * g], ov[2 * g + 1] - ov[2 * g]);
                pos = ov[1] == ov[0] ? ov[1] + 1 : ov[1];
            }
            return o;
        });
        report("pcre2-jit", c.id, out, best, text.size(), compile * 1e6);
        pcre2_match_data_free(md);
        pcre2_code_free(code);
    }
}
