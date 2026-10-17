main :: ()
{
    if x
    {
        a();
    }
    else
    {
        b();
    }
    if y { c(); }
    else { d(); }
    for 0..3
    {
    }
    // A block statement after a statement stays on its own line.
    e();
    {
        f();
    }
}

#if OS == .WINDOWS {
    #load "windows.jai";
}
else {
    #load "unix.jai";
}
