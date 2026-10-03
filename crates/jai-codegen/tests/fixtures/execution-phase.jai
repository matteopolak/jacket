phase :: () -> bool #no_context { return #compile_time; }
OBSERVED :: #run phase();
counter:int=0;
increment :: () -> bool { counter+=1; return true; }
main :: () -> int {
    if !OBSERVED return 1;
    answer := ifx increment() && #compile_time then 41 else 42;
    if counter!=1 return 2;
    if phase() return 41;
    return answer;
}
