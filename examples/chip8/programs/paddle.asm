; Keep the falling ball above the floor. Q/E move the paddle; a miss shows F.
; V0 paddle x, V1 paddle y, V2 ball x, V3 ball y, V4 scratch, V5 key.
op 601C
op 611D
op C23F
op 6300
frame:
op 00E0
index paddle
op D013
index ball
op D231
op 6402
op F415
wait:
op F407
op 3400
jp wait
op 6504
op E5A1
op 70FF
op 6506
op E5A1
op 7001
op 7301
op 331C
jp frame
; Horizontal distance ball-paddle < 8 means a hit.
op 8420
op 8405
op 6608
op 8465
op 3F00
jp miss
op 6404
op F418
op C23F
op 6300
jp frame
miss:
op 00E0
op 600F
op F029
op 6100
op D015
stop:
jp stop
paddle:
byte FF
byte 81
byte FF
byte 00
ball:
byte 80
