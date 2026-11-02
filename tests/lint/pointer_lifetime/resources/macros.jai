#import "Basic";

free_and_read :: (pointer: *int) #expand {
    free(pointer);
    print("%\n", pointer.*);
}
