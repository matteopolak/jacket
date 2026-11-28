// Decodes MPEG-1 program streams: MPEG-1 video and MPEG-1 Layer II (MP2) audio.
//
// Create a [Plm] with [create_with_filename], [create_with_file], [create_with_memory] or [create_with_buffer].
// Then either call [decode] with elapsed seconds and receive frames through [set_video_decode_callback] and
// [set_audio_decode_callback], or pull them with [decode_video] and [decode_audio]. [seek] jumps to a time and
// [plm_destroy] releases the player.
//
// Video arrives as a [Frame] of YCbCr planes, which [frame_to_rgb] and its siblings convert to packed pixels.
// Audio arrives as [Samples] of interleaved stereo floats. Frames and samples belong to the decoder and are
// overwritten by the next decode. Malformed or unsupported streams assert.
// Streams can also be fed incrementally through a [Buffer] made with [buffer_create_with_capacity].

#load "../internal/media/mpeg/buffer.jai";
#load "../internal/media/mpeg/pixels.jai";
#load "../internal/media/mpeg/tables.jai";
#load "../internal/media/mpeg/idct.jai";
#load "../internal/media/mpeg/video.jai";
#load "../internal/media/mpeg/motion.jai";
#load "../internal/media/mpeg/reconstruction.jai";
#load "../internal/media/mpeg/synthesis.jai";
#load "../internal/media/mpeg/audio.jai";
#load "../internal/media/mpeg/program.jai";
