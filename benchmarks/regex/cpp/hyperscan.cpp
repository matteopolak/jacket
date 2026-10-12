// Hyperscan / vectorscan harness (block mode). Hyperscan reports every match end instead of
// leftmost-first non-overlapping matches, so with start-of-match reporting a match is counted
// only when it starts at or after the end of the previously counted one. No capture groups.
#include <hs/hs.h>

#include "common.h"

struct Ctx {
    uint64_t count = 0;
    unsigned long long last_end = 0;
};

static int on_match(unsigned, unsigned long long from, unsigned long long to, unsigned, void* p) {
    Ctx* c = (Ctx*)p;
    if (from >= c->last_end && to > from) {
        c->count++;
        c->last_end = to;
    }
    return 0;
}

int main(int argc, char** argv) {
    Harness h = parse_args(argc, argv);
    const std::string& text = h.corpus;
    const unsigned flags = HS_FLAG_UTF8 | HS_FLAG_UCP | HS_FLAG_SOM_LEFTMOST;
    for (const Case& c : h.cases) {
        if (c.id == "caps") {
            report_na("hyperscan", c.id, "no capture groups");
            continue;
        }
        hs_database_t* db = nullptr;
        double compile = 1e30;
        std::string why, note;
        // Try Unicode classes (UCP) first; some patterns are unsupported or too large with
        // them, and then the ASCII-class version is measured instead, with a note.
        for (unsigned f : {flags, flags & ~(unsigned)HS_FLAG_UCP}) {
            std::string first_why = why;
            for (int i = 0; i < 20; i++) {
                double t = now_seconds();
                hs_database_t* d = nullptr;
                hs_compile_error_t* e = nullptr;
                if (hs_compile(c.pattern.c_str(), f, HS_MODE_BLOCK, nullptr, &d, &e) != HS_SUCCESS) {
                    why = e->message;
                    hs_free_compile_error(e);
                    break;
                }
                compile = std::min(compile, now_seconds() - t);
                if (db) hs_free_database(db);
                db = d;
            }
            if (db) {
                if (!(f & HS_FLAG_UCP)) note = "UCP unavailable (" + first_why + "), ASCII classes";
                break;
            }
        }
        if (!db) {
            report_na("hyperscan", c.id, "compile error: " + why);
            continue;
        }
        hs_scratch_t* scratch = nullptr;
        hs_alloc_scratch(db, &scratch);
        Outcome out;
        double best = best_of(h.runs, out, [&]() {
            Ctx ctx;
            hs_scan(db, text.data(), (unsigned)text.size(), 0, scratch, on_match, &ctx);
            return Outcome{ctx.count, 0};
        });
        report("hyperscan", c.id, out, best, text.size(), compile * 1e6, note);
        hs_free_scratch(scratch);
        hs_free_database(db);
    }
}
