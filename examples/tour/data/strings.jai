// Strings are UTF-8 byte views; `print` formats anything.

tour_strings :: () {
    motto := "Simple, fast, and fun";
    // A string is a count plus a pointer, so slicing is free.
    first_word := slice(motto, 0, 6);
    first_byte := string.{ 1, motto.data };
    print("\"%\" has % bytes; the first word is \"%\" and byte 0 is '%'\n",
        motto, motto.count, first_word, first_byte);

    // `%` takes the next argument; `%1`, `%2` pick arguments by position.
    print("%1, %1. %2\n", "Knock", "Who's there?");

    // Formatting helpers wrap a value with options.
    pi := 3.14159265;
    print("pi to 2 places: %, padded: [%]\n",
        formatFloat(pi, trailing_width = 2), formatFloat(pi, width = 10, trailing_width = 4));
    print("255 in hex: %, binary: %; 7 padded: %\n",
        formatInt(255, base = 16), formatInt(255, base = 2), formatInt(7, minimum_digits = 3));
    big := formatInt(1234567890, digits_per_comma = 3, comma_string = ",");
    print("big numbers get commas: %\n", big);

    // Structs and arrays print themselves.
    print("a struct: %, an array: %\n", Vec2.{ 3, 4 }, int.[1, 2, 3]);

    // String_Builder gathers pieces without reallocating each time.
    builder: String_Builder;
    for 1..15 {
        if it % 15 == 0 append(*builder, "FizzBuzz");
        else if it % 3 == 0 append(*builder, "Fizz");
        else if it % 5 == 0 append(*builder, "Buzz");
        else print_to_builder(*builder, "%", it);
        if it < 15 append(*builder, " ");
    }
    fizzbuzz := builder_to_string(*builder);
    defer free(fizzbuzz);
    print("%\n", fizzbuzz);

    // Some help from the String module, allocating from temporary storage.
    {
        push_allocator(temp);
        parts := split(motto, ", ");
        joined := join(..parts, separator = " + ");
        print("upper: %; split: %; joined: %\n", to_upper_copy(motto), parts, joined);
    }
    print("contains \"fast\": %, starts with \"Simple\": %\n",
        contains(motto, "fast"), begins_with(motto, "Simple"));

    // Here-strings hold multi-line text verbatim until the closing token.
    banner := #string DONE
   _       _
  (_) __ _(_)
  | |/ _` | |
  | | (_| | |
 _/ |\__,_|_|
|__/
DONE
    print("%", banner);
}
