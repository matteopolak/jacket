; Original self-test. On success draw A; on failure draw F.
op 00E0
op 6105
op 6207
op 8124
op 310C
jp fail
op 8125
op 3105
jp fail
op 8127
op 3102
jp fail
op 8116
op 3101
jp fail
op 811E
op 3102
jp fail
call sub
op 3103
jp fail
op 600A
jp show
fail:
op 600F
show:
op F029
op 6100
op 6200
op D125
op 00FD
sub:
op 7101
op 00EE
