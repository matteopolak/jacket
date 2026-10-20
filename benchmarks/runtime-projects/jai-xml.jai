// XML parsing, walking and writing with smari/jai-xml over a generated document.
// Copied into the project as its entry.
#import "Basic";
#import,file "module.jai"(USE_SIMD_STRING_SEARCH = false);

main :: () {
    builder: String_Builder;

    append(*builder, "<?xml version=\"1.0\"?>\n<catalog>\n");

    // Shelves of 50 books: the library walks siblings recursively, so long sibling lists need deep stacks.
    for shelf: 0..800 - 1 {
        print_to_builder(*builder, "  <shelf n=\"%\">\n", shelf);

        for book: 0..50 - 1 {
            it := shelf * 50 + book;
            print_to_builder(*builder, "    <book id=\"b%\" year=\"%\">\n      <title>Book number %</title>\n      <author>Author %</author>\n      <price>%.%</price>\n      <!-- note % -->\n    </book>\n",
                             it, 1900 + it % 120, it, it % 97, it % 50, 10 + it % 89, it);
        }

        append(*builder, "  </shelf>\n");
    }

    append(*builder, "</catalog>\n");
    text := builder_to_string(*builder);

    for round: 0..2 {
        root, result := xml_parse(text);

        assert(result.status == .ok);
        nodes := 0;
        for root nodes += 1;

        written := xml_write(root);

        print("% % %\n", text.count, nodes, written.count);
        xml_free(root);
    }
}
