target_only :: #library,system "NoSuchTargetFrameworkForTests";
target_only_call :: () -> s32 #foreign target_only;

#run target_only_call();
