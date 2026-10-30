// Bit_Array: a fixed-length sequence of bits packed into words.

// A fixed-length sequence of bits packed into 64-bit words; `count` is the number of bits.
//
// The words live in `slots`, allocated from `allocator` (`context.allocator` when unset) by
// `init_bit_array` or `make_bit_array`, and released by `deinit`. Bits start cleared. Bit `i` is read with
// `bits[i]` and written with `bits[i] = value`; an index outside `0..count-1` fails an assert. The length is
// fixed once made: there is no growing.
//
// When `count` is not a multiple of 64, the unused high bits of the last word are padding. They are not
// reachable through the indexing or iteration, but `set_all_bits` and `toggle_all_bits` change them too, so
// compare `slots` word by word only for lengths that fill whole words.
//
// ```jai
// bits := make_bit_array(10);
// set_bit(*bits, 2);
// bits[5] = true;
// toggle_bit(*bits, 2);
//
// assert(!bits[2] && bits[5]);
//
// set := 0;
// for bits if it set += 1; // `it` is the bit, `it_index` its position
// assert(set == 1);
// deinit(*bits);
// ```
Bit_Array :: Impl.Bit_Array;
// Sets up `array` to hold `count` bits, all cleared, replacing whatever it was. Allocates the words with the array's allocator, taking `context.allocator` if it has none.
//
// Use it on a `Bit_Array` that holds no storage, such as a zeroed one or one after `deinit`; storage it already
// owned would leak. A negative `count` fails an assert.
init_bit_array :: Impl.init_bit_array;
// A new `Bit_Array` of `count` cleared bits. The caller owns the storage and releases it with `deinit`.
//
// ```jai
// bits := make_bit_array(200);
// assert(bits.count == 200 && !bits[199]);
// deinit(*bits);
// ```
make_bit_array :: Impl.make_bit_array;
// Frees the words and leaves the array empty (`count` is zero). It can be set up again with `init_bit_array`.
deinit :: Impl.deinit;
// Sets bit `i`; `bits[i]` reads it back.
//
// ```jai
// bits := make_bit_array(8);
// set_bit(*bits, 3);
// assert(bits[3] && !bits[2]);
// ```
set_bit :: Impl.set_bit;
// Clears bit `i`. An index outside `0..count-1` fails an assert.
//
// ```jai
// bits := make_bit_array(4);
// set_bit(*bits, 1);
// clear_bit(*bits, 1);
// assert(!bits[1]);
// deinit(*bits);
// ```
clear_bit :: Impl.clear_bit;
// Flips bit `i`. An index outside `0..count-1` fails an assert.
toggle_bit :: Impl.toggle_bit;
// Sets bit `i` when `value` is `true` and clears it otherwise; this is what `bits[i] = value` does.
set_bit_to :: Impl.set_bit_to;
// Clears every bit, including the padding bits of the last word.
//
// ```jai
// bits := make_bit_array(70);
// set_all_bits(*bits);
// clear_all_bits(*bits);
// for bits assert(!it);
// deinit(*bits);
// ```
clear_all_bits :: Impl.clear_all_bits;
// Sets every bit, including the padding bits of the last word, so `slots` holds all ones.
//
// ```jai
// bits := make_bit_array(5);
// set_all_bits(*bits);
// for bits assert(it);
// deinit(*bits);
// ```
set_all_bits :: Impl.set_all_bits;
// Flips every bit, including the padding bits of the last word.
toggle_all_bits :: Impl.toggle_all_bits;
// Lets a `Bit_Array` be used in a `for` loop over all its bits, in order: `it` is the bit and `it_index` its position.
//
// Pointer and reverse loops (`for *`, `for <`) are not supported and fail an assert. See `only_set` and
// `only_unset` to skip the others.
for_expansion :: Impl.for_expansion;
// A loop body for visiting just the set bits, used as `for: only_set bits`; `it` is `true` and `it_index` the position.
//
// ```jai
// bits := make_bit_array(8);
// bits[1] = true;
// bits[6] = true;
//
// positions: [..] int;
// for: only_set bits array_add(*positions, it_index);
// assert(positions.count == 2 && positions[1] == 6);
// array_free(positions);
// deinit(*bits);
// ```
only_set :: Impl.only_set;
// Like `only_set`, but visits the cleared bits.
only_unset :: Impl.only_unset;

using,only(.["[]", "[]="]) #import "internal/collections/bit-array";

#scope_file
Impl :: #import "internal/collections/bit-array";
