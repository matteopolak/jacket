#import "Basic";
G :: #import "developer-tools-public-bridge-generated";

#if OS == .WINDOWS #assert(type_of(G.NativeBridge_choose(5)) == s32);
else #assert(type_of(G.NativeBridge_choose(5)) == s64);

main :: () {
    assert(G.NativeBridge_deep(null) == null);
    assert(G.global_cpp() == 42 && G.global_cpp(5) == 43);
    assert(G.NativeBridge_sum(5.75) == 12 && G.NativeBridge_sum(5, 9) == 14);
    assert(G.NativeBridge_choose(5) == 105 && G.NativeBridge_choose_1(5) == 205);
    pair: G.Pair;

    G.NativeBridge_make_pair(*pair, 18);
    assert(pair.first == 18 && pair.second == 19);
    shared := G.NativeBridge_shared_value();

    assert(shared.* == 9);
    shared.* = 23;
    assert(G.NativeBridge_shared_value().* == 23);
    assert(G.NativeBridge_bump(shared) == 27 && shared.* == 27);
    values := s32.[2, 3, 4];

    assert(G.NativeBridge_sum_array(values.data) == 9);
    matrix: [2][3] s32;
    for row: 0..1 for column : 0..2 matrix[row][column] = cast(s32) (row * 3 + column + 1);

    assert(G.NativeBridge_sum_matrix(*matrix[0]) == 21);
    bytes := u8.[65, 0, 66];

    assert(G.NativeBridge_message_message(3, bytes.data) == 131);
    print("ok\n");
}
