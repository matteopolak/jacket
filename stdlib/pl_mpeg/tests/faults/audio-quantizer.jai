M :: #import "pl_mpeg";

main :: () {
    bytes := u8.[0xff];
    audio: M.Audio;
    audio.buffer = M.buffer_create_with_memory(bytes, false);
    table := *M.QUANT_TAB;

    audio.allocation[0][0] = table.data;
    M.audio_read_samples(*audio, 0, 0, 0);
}
