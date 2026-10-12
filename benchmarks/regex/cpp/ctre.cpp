// CTRE harness: every pattern is compiled by the C++ compiler (header-only, C++20). Input is
// UTF-8 (std::u8string_view) with <ctre-unicode.hpp>, so \p{Greek} works, but CTRE's \w and \b
// are ASCII only. Compile time is the C++ build, so the compile column is 0.
#include <ctre-unicode.hpp>

#include "common.h"

using namespace ctre::literals;

#define PLAIN_CASE(ID, PAT)                                                                      \
    if (want(ID)) {                                                                              \
        Outcome out;                                                                             \
        double best = best_of(h.runs, out, [&]() {                                               \
            Outcome o;                                                                           \
            for (auto m : ctre::search_all<PAT>(sv)) { (void)m; o.count++; }                     \
            return o;                                                                            \
        });                                                                                      \
        report("ctre", ID, out, best, text.size(), 0);                                           \
    }

int main(int argc, char** argv) {
    Harness h = parse_args(argc, argv);
    const std::string& text = h.corpus;
    std::u8string_view sv((const char8_t*)text.data(), text.size());
    auto want = [&](const char* id) {
        for (const Case& c : h.cases) if (c.id == id) return true;
        return false;
    };
    PLAIN_CASE("date", "[0-9]{4}-[0-9]{2}-[0-9]{2}")
    PLAIN_CASE("email", "\\w+@\\w+\\.com")
    PLAIN_CASE("greek", "\\p{Greek}+")
    PLAIN_CASE("nomatch", "zzz")
    PLAIN_CASE("nocase", "(?i)HELLO")
    PLAIN_CASE("wordb", "\\b\\w+\\b")
    PLAIN_CASE("alt", "Sherlock|Holmes|Watson|Irene|Adler")
    PLAIN_CASE("literal", "the")
    PLAIN_CASE("ing", "[a-zA-Z]+ing")
    if (want("caps")) {
        Outcome out;
        double best = best_of(h.runs, out, [&]() {
            Outcome o;
            for (auto m : ctre::search_all<"([0-9]{4})-([0-9]{2})-([0-9]{2})">(sv)) {
                o.count++;
                o.check += to_u64((const char*)m.get<1>().to_view().data(), 4) + to_u64((const char*)m.get<2>().to_view().data(), 2) +
                           to_u64((const char*)m.get<3>().to_view().data(), 2);
            }
            return o;
        });
        report("ctre", "caps", out, best, text.size(), 0);
    }
}
