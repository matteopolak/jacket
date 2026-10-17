R :: #import "Record";
F :: #import "Factory";
M :: #import "Mutation";

#assert R.Buffer == R.Alias;
#assert R.Buffer != F.Buffer;
#assert type_of(R.create()) == *R.Alias;

#program_export "namespace_check"
check :: () -> s32 #c_call {
    M.values[0] = 9;
    M.record.value = 7;
    M.pointer.value += 4;
    if M.values[0] != 9 || M.record.value != 11 return 3;

    push_context {
        value := R.create();
        if value.value != 37 || value.next != null return 1;

        value.next = value;
        again := R.create();
        if again != value || again.value != 37 || again.next != null return 2;
    }

    return 0;
}
