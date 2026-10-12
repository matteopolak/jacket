// RE2 harness (run-time compile). Unicode: UTF-8 input, but RE2's \w and \b are ASCII only.
#include <re2/re2.h>

#include "common.h"

int main(int argc, char** argv) {
    Harness h = parse_args(argc, argv);
    const std::string& text = h.corpus;
    RE2::Options opts;
    opts.set_max_mem(512 << 20);
    for (const Case& c : h.cases) {
        double compile = 1e30;
        for (int i = 0; i < 20; i++) {
            double t = now_seconds();
            RE2 r(c.pattern, opts);
            compile = std::min(compile, now_seconds() - t);
        }
        RE2 re(c.pattern, opts);
        if (!re.ok()) {
            report_na("re2", c.id, "compile error: " + re.error());
            continue;
        }
        bool caps = c.id == "caps";
        re2::StringPiece sp(text);
        Outcome out;
        double best = best_of(h.runs, out, [&]() {
            Outcome o;
            size_t pos = 0;
            re2::StringPiece m[4];
            int n = caps ? 4 : 1;
            while (pos <= text.size() && re.Match(sp, pos, text.size(), RE2::UNANCHORED, m, n)) {
                o.count++;
                if (caps) for (int g = 1; g <= 3; g++) o.check += to_u64(m[g].data(), m[g].size());
                size_t end = (size_t)(m[0].data() - text.data()) + m[0].size();
                pos = m[0].size() == 0 ? end + 1 : end;
            }
            return o;
        });
        report("re2", c.id, out, best, text.size(), compile * 1e6);
    }
}
