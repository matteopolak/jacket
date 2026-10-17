#import "Basic";
#import "Sound_Player"; // jailint: allow(unused_import) only the Windows branch uses it

main :: () {
    #if OS == .WINDOWS {
        assert(CONFIG(0x12340004) == .STEREO && GEOMETRY(0x12340004) == 0x34);
        assert(KSAUDIO_SPEAKER_STEREO == 3 && KSAUDIO_SPEAKER_5POINT1 == 0x3f);
        #assert size_of(WAVEFORMATEXTENSIBLE) == 40;
        #assert size_of(DSBUFFERDESC) == 40;
        #assert size_of(IDirectSound8Vtable) == 11 * size_of(*void);
        #assert size_of(IDirectSound8) == size_of(*void);
        #assert size_of(IDirectSoundBuffer8Vtable) == 24 * size_of(*void);
        format: WAVEFORMATEXTENSIBLE;

        init_wave_format(*format.Format, 2);
        assert(format.Format.nChannels == 2 && format.Format.nSamplesPerSec == 44100);
        assert(format.Format.nBlockAlign == 4 && format.Format.nAvgBytesPerSec == 176400);
        init_KSDATAFORMAT_SUBTYPE_PCM(*format.SubFormat);
        assert(format.SubFormat.Data1 == 1 && format.SubFormat.Data3 == 0x10);
    }

    print("ok\n");
}
