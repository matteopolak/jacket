worker :: () -> int { return 42; }
alias :: worker;

main :: () -> int {
    if #procedure_name() != "main" return 1;
    if #procedure_name(#this) != "main" return 2;
    if #procedure_name(worker) != "worker" return 3;
    if #procedure_name(alias) != "worker" return 4;
    return alias();
}
