// Canonical high-level playback record. Private demux state is defined below.
Plm :: struct {
    demux: *Demux;
    time: float64;

    has_ended: bool;
    loop: bool;
    has_decoders: bool;
    video_enabled := true;
    video_packet_type: Start_Code;

    video_buffer: *Buffer;
    video_decoder: *Video;

    audio_enabled := true;
    audio_stream_index: s32;
    audio_packet_type: Start_Code;

    audio_lead_time: float64;
    audio_buffer: *Buffer;
    audio_decoder: *Audio;

    video_decode_callback: Video_Decode_Callback;
    video_decode_callback_user_data: *void;

    audio_decode_callback: Audio_Decode_Callback;
    audio_decode_callback_user_data: *void;
}

Video_Decode_Callback :: #type (self: *Plm, frame: *Frame, user: *void);
Audio_Decode_Callback :: #type (plm: *Plm, samples: *Samples, user: *void);
INVALID_TS: float64 : -1.0;

Packet :: struct {
    type: Start_Code;
    pts: float64;
    data: [] u8;
}

create_with_filename :: (filename: string) -> *Plm {
    buffer := buffer_create_with_filename(filename);
    if !buffer return null;
    return create_with_buffer(buffer, true);
}

create_with_file :: (fh: File, close_when_done: bool) -> *Plm {
    buffer := buffer_create_with_file(fh, close_when_done);
    if !buffer return null;
    return create_with_buffer(buffer, true);
}

create_with_memory :: (memory: [] u8, free_when_done: bool) -> *Plm {
    buffer := buffer_create_with_memory(memory, free_when_done);
    if !buffer return null;
    return create_with_buffer(buffer, true);
}

create_with_buffer :: (buffer: *Buffer, destroy_when_done: bool) -> *Plm {
    if !buffer return null;

    self := B.New(Plm);
    self.demux = B.New(Demux);
    self.demux.source = buffer;
    self.demux.destroy_source = destroy_when_done;
    self.demux.code = .NOT_FOUND;
    initialize_decoders(self);

    return self;
}

plm_destroy :: (using self: *Plm) {
    if !self return;
    if video_decoder video_destroy(video_decoder);
    else if video_buffer buffer_destroy(video_buffer);
    if audio_decoder audio_destroy(audio_decoder);
    else if audio_buffer buffer_destroy(audio_buffer);

    if demux {
        if demux.destroy_source buffer_destroy(demux.source);
        B.free(demux);
    }

    B.free(self);
} @Releases

has_headers :: (using self: *Plm) -> bool {
    B.assert(self != null && demux != null);
    if !initialize_decoders(self) return false;
    if video_decoder && !video_has_header(video_decoder) return false;
    if audio_decoder && !audio_has_header(audio_decoder) return false;

    return true;
}

get_num_video_streams :: (self: *Plm) -> s32 {
    B.assert(self != null);
    if !demux_headers(self.demux) return 0;
    return self.demux.video_streams;
}

get_size :: (self: *Plm) -> (width: s32, height: s32) {
    if initialize_decoders(self) && self.video_decoder && video_has_header(self.video_decoder) {
        return self.video_decoder.width, self.video_decoder.height;
    }

    return 0, 0;
}

get_framerate :: (self: *Plm) -> float64 {
    if initialize_decoders(self) && self.video_decoder return video_get_framerate(self.video_decoder);
    return 0;
}

get_samplerate :: (self: *Plm) -> s32 {
    if initialize_decoders(self) && self.audio_decoder && audio_has_header(self.audio_decoder) return SAMPLE_RATE[self.audio_decoder.samplerate_index];
    return 0;
}

set_audio_enabled :: (using self: *Plm, enabled: bool) {
    B.assert(self != null);
    audio_enabled = enabled;
    audio_packet_type = .PICTURE;
    if enabled && initialize_decoders(self) && audio_decoder audio_packet_type = cast(Start_Code) (cast(s32) Start_Code.AUDIO_1 + audio_stream_index);
}

set_video_enabled :: (using self: *Plm, enabled: bool) {
    B.assert(self != null);
    video_enabled = enabled;
    video_packet_type = .PICTURE;
    if enabled && initialize_decoders(self) && video_decoder video_packet_type = .VIDEO_1;
}

set_audio_stream :: (self: *Plm, stream_index: s32) {
    B.assert(self != null && stream_index >= 0 && stream_index < 4, "MPEG audio stream index out of range");
    self.audio_stream_index = stream_index;
    set_audio_enabled(self, self.audio_enabled);
}

set_loop :: (self: *Plm, loop: bool) {
    B.assert(self != null);
    self.loop = loop;
}

set_video_decode_callback :: (self: *Plm, callback: Video_Decode_Callback, user: *void) {
    B.assert(self != null);
    self.video_decode_callback = callback;
    self.video_decode_callback_user_data = user;
}

set_audio_decode_callback :: (self: *Plm, callback: Audio_Decode_Callback, user: *void) {
    B.assert(self != null);
    self.audio_decode_callback = callback;
    self.audio_decode_callback_user_data = user;
}

rewind :: (using self: *Plm) {
    B.assert(self != null && demux != null);
    if video_decoder video_rewind(video_decoder);
    if audio_decoder audio_rewind(audio_decoder);

    buffer_rewind(demux.source);
    demux.code = .NOT_FOUND;
    time = 0;
    has_ended = false;
}

handle_end :: (self: *Plm) {
    B.assert(self != null);
    if self.loop rewind(self);
    else self.has_ended = true;
}

decode_video :: (self: *Plm) -> *Frame {
    if !initialize_decoders(self) || self.video_packet_type == .PICTURE return null;

    frame := video_decode(self.video_decoder);
    if frame self.time = frame.time;
    else if self.demux.source.has_ended handle_end(self);

    return frame;
}

decode_audio :: (self: *Plm) -> *Samples {
    if !initialize_decoders(self) || self.audio_packet_type == .PICTURE return null;

    samples := audio_decode(self.audio_decoder);
    if samples self.time = samples.time;
    else if self.demux.source.has_ended handle_end(self);

    return samples;
}

decode :: (self: *Plm, tick: float64) {
    B.assert(self != null && tick >= 0 && tick <= 1.7976931348623157e308, "invalid MPEG playback tick");
    if !initialize_decoders(self) return;

    want_video := self.video_decoder != null && self.video_packet_type != .PICTURE && self.video_decode_callback != null;
    want_audio := self.audio_decoder != null && self.audio_packet_type != .PICTURE && self.audio_decode_callback != null;

    if !want_video && !want_audio return;

    video_target := self.time + tick;
    audio_target := video_target + self.audio_lead_time;

    B.assert(video_target <= 1.7976931348623157e308 && audio_target <= 1.7976931348623157e308);
    failed_video := false;
    failed_audio := false;

    while true {
        progressed := false;

        if want_video && self.video_decoder.time < video_target {
            frame := video_decode(self.video_decoder);
            if frame {
                self.video_decode_callback(self, frame, self.video_decode_callback_user_data);
                progressed = true;
            } else failed_video = true;
        }

        if want_audio && self.audio_decoder.time < audio_target {
            samples := audio_decode(self.audio_decoder);
            if samples {
                self.audio_decode_callback(self, samples, self.audio_decode_callback_user_data);
                progressed = true;
            } else failed_audio = true;
        }

        if !progressed break;
    }

    if (!want_video || failed_video) && (!want_audio || failed_audio) && self.demux.source.has_ended {
        handle_end(self);
        return;
    }

    self.time = video_target;
}

read_video_packet :: (buffer: *Buffer, user: *void) {
    self := cast(*Plm) user;
    B.assert(self != null && buffer == self.video_buffer);
    read_packets(self, self.video_packet_type);
}

read_audio_packet :: (buffer: *Buffer, user: *void) {
    self := cast(*Plm) user;
    B.assert(self != null && buffer == self.audio_buffer);
    read_packets(self, self.audio_packet_type);
}

read_packets :: (self: *Plm, requested_type: Start_Code) {
    B.assert(self != null && self.demux != null);

    while true {
        packet := demux_next(self.demux);
        if !packet break;

        if packet.type == self.video_packet_type && self.video_buffer {
            written := buffer_write(self.video_buffer, packet.data);
            if written != packet.data.count {
                context.logger("MPEG video buffer allocation failed\n", context.logger_data, Log_Info.{ common_flags = .ERROR });
                B.assert(false, "MPEG video buffer allocation failed");
                return;
            }
        } else if packet.type == self.audio_packet_type && self.audio_buffer {
            written := buffer_write(self.audio_buffer, packet.data);
            if written != packet.data.count {
                context.logger("MPEG audio buffer allocation failed\n", context.logger_data, Log_Info.{ common_flags = .ERROR });
                B.assert(false, "MPEG audio buffer allocation failed");
                return;
            }
        }

        if packet.type == requested_type return;
    }

    if self.demux.source.has_ended {
        if self.video_buffer buffer_signal_end(self.video_buffer);
        if self.audio_buffer buffer_signal_end(self.audio_buffer);
    }
}

// Linear seeks retain real decoded frames and work without a side index. The
// precise/nonprecise frame selection and audio alignment require contract review.
seek_frame :: (self: *Plm, time: float64, seek_exact: bool) -> *Frame {
    B.assert(self != null && time == time && time >= -1.7976931348623157e308 && time <= 1.7976931348623157e308);
    if !initialize_decoders(self) || self.video_packet_type == .PICTURE return null;

    B.assert(self.demux.source.mode != .RING, "MPEG seek requires a seekable source");
    target := time;
    if target < 0 target = 0;

    old_loop := self.loop;

    self.loop = false;
    defer self.loop = old_loop;
    rewind(self);
    frame: *Frame;

    while true {
        next := decode_video(self);
        if !next break;

        frame = next;

        if seek_exact {
            if frame.time >= target break;
        } else if frame.time + 1.0 / self.video_decoder.framerate > target break;
    }

    if frame {
        self.time = frame.time;
        self.has_ended = false;
    }

    return frame;
}

seek :: (self: *Plm, time: float64, seek_exact: bool) -> bool {
    frame := seek_frame(self, time, seek_exact);
    if !frame return false;
    if self.video_decode_callback self.video_decode_callback(self, frame, self.video_decode_callback_user_data);

    if self.audio_decoder && self.audio_packet_type != .PICTURE {
        while self.audio_decoder.time < self.time {
            if !audio_decode(self.audio_decoder) break;
        }
    }

    return true;
}

#scope_file
B :: #import "Basic";
#import "File";

Demux :: struct {
    source: *Buffer;
    destroy_source: bool;
    has_pack, has_system: bool;
    video_streams, audio_streams: s32;
    code: Start_Code;
    packet: Packet;
}

initialize_decoders :: (self: *Plm) -> bool {
    B.assert(self != null && self.demux != null);
    if self.has_decoders return true;
    if !demux_headers(self.demux) return false;

    if self.demux.video_streams > 0 {
        if self.video_enabled self.video_packet_type = .VIDEO_1;

        if !self.video_buffer {
            self.video_buffer = buffer_create_with_capacity(BUFFER_DEFAULT_SIZE);
            if !self.video_buffer return false;
            buffer_set_load_callback(self.video_buffer, read_video_packet, self);
        }

        if !self.video_decoder self.video_decoder = video_create_with_buffer(self.video_buffer, true);
    }

    if self.demux.audio_streams > 0 {
        if self.audio_enabled self.audio_packet_type = cast(Start_Code) (cast(s32) Start_Code.AUDIO_1 + self.audio_stream_index);

        if !self.audio_buffer {
            self.audio_buffer = buffer_create_with_capacity(BUFFER_DEFAULT_SIZE);
            if !self.audio_buffer return false;
            buffer_set_load_callback(self.audio_buffer, read_audio_packet, self);
        }

        if !self.audio_decoder self.audio_decoder = audio_create_with_buffer(self.audio_buffer, true);
    }

    self.has_decoders = true;

    return true;
}

demux_headers :: (demux: *Demux) -> bool {
    if demux.has_pack && demux.has_system return true;

    buffer := demux.source;

    while !demux.has_pack || !demux.has_system {
        if demux.code == .NOT_FOUND demux.code = buffer_next_start_code(buffer);
        if demux.code == .NOT_FOUND return false;

        if demux.code == .PACK {
            if !parse_pack(buffer) return false;
            demux.has_pack = true;
        } else if demux.code == .SYSTEM {
            if !parse_system(demux) return false;
            demux.has_system = true;
        }

        demux.code = .NOT_FOUND;
    }

    return true;
}

parse_pack :: (buffer: *Buffer) -> bool {
    if !buffer_has(buffer, 64) return false;

    if !read_pack_field(buffer, 4, 2, "unsupported MPEG pack version") return false;

    buffer_skip(buffer, 3);
    if !read_pack_field(buffer, 1, 1, "invalid MPEG pack marker") return false;

    buffer_skip(buffer, 15);
    if !read_pack_field(buffer, 1, 1, "invalid MPEG pack marker") return false;

    buffer_skip(buffer, 15);
    if !read_pack_field(buffer, 1, 1, "invalid MPEG pack marker") return false;
    if !read_pack_field(buffer, 1, 1, "invalid MPEG pack marker") return false;

    buffer_skip(buffer, 22);
    if !read_pack_field(buffer, 1, 1, "invalid MPEG pack marker") return false;

    return true;
}

read_pack_field :: (buffer: *Buffer, count: s64, expected: s32, message: string) -> bool {
    value := buffer_read(buffer, count);
    if value == expected return true;

    context.logger(message, context.logger_data, Log_Info.{ common_flags = .ERROR });
    B.assert(false, message);

    return false;
}

parse_system :: (demux: *Demux) -> bool {
    buffer := demux.source;
    if buffer.discard_read_bytes buffer_discard_read_bytes(buffer);

    discard := buffer.discard_read_bytes;

    buffer.discard_read_bytes = false;
    saved := buffer.bit_index;
    committed := false;

    defer {
        buffer.discard_read_bytes = discard;
        if !committed buffer.bit_index = saved;
    }

    if !buffer_has(buffer, 16) return false;

    length := buffer_read(buffer, 16);

    B.assert(length >= 6, "invalid MPEG system header length");
    if !buffer_has(buffer, cast(s64) length * 8) return false;

    buffer_skip(buffer, 24);
    demux.audio_streams = buffer_read(buffer, 6);
    buffer_skip(buffer, 5);
    demux.video_streams = buffer_read(buffer, 5);
    buffer_skip(buffer, cast(s64) (length - 5) * 8);
    committed = true;

    return true;
}

demux_next :: (demux: *Demux) -> *Packet {
    if !demux_headers(demux) return null;

    buffer := demux.source;

    while true {
        if demux.code == .NOT_FOUND demux.code = buffer_next_start_code(buffer);
        if demux.code == .NOT_FOUND return null;

        if demux.code == .PACK {
            if !parse_pack(buffer) return null;
            demux.code = .NOT_FOUND;
            continue;
        }

        if demux.code == .SYSTEM {
            if !parse_system(demux) return null;
            demux.code = .NOT_FOUND;
            continue;
        }

        if demux.code == .END {
            buffer.has_ended = true;
            demux.code = .NOT_FOUND;
            return null;
        }

        type := demux.code;
        if cast(u8) type < 0xbc {
            demux.code = .NOT_FOUND;
            continue;
        }

        if buffer.discard_read_bytes buffer_discard_read_bytes(buffer);

        discard := buffer.discard_read_bytes;

        buffer.discard_read_bytes = false;
        saved := buffer.bit_index;

        if !buffer_has(buffer, 16) {
            buffer.discard_read_bytes = discard;
            return null;
        }

        length := buffer_read(buffer, 16);

        B.assert(length > 0, "unbounded MPEG PES packets are unsupported");

        if !buffer_has(buffer, cast(s64) length * 8) {
            buffer.bit_index = saved;
            buffer.discard_read_bytes = discard;
            B.assert(!buffer.has_ended, "truncated MPEG PES packet");
            return null;
        }

        first := buffer.bit_index >> 3;
        end := first + length;

        buffer.bit_index = end * 8;
        buffer.discard_read_bytes = discard;
        demux.code = .NOT_FOUND;
        if type != .VIDEO_1 && type != .PRIVATE && (type < .AUDIO_1 || type > .AUDIO_4) continue;
        while first < end && buffer.bytes[first] == 0xff first += 1;

        B.assert(first < end, "missing MPEG PES header");

        if (buffer.bytes[first] & 0xc0) == 0x40 {
            first += 2;
            B.assert(first < end, "truncated MPEG P-STD header");
        }

        marker := buffer.bytes[first] >> 4;
        pts := INVALID_TS;

        if marker == 2 || marker == 3 {
            B.assert(end - first >= 5, "truncated MPEG PTS");
            pts = decode_pts(buffer.bytes + first, marker);
            first += 5;

            if marker == 3 {
                B.assert(end - first >= 5, "truncated MPEG DTS");
                decode_pts(buffer.bytes + first, 1);
                first += 5;
            }
        } else {
            B.assert(buffer.bytes[first] == 0x0f, "unsupported MPEG PES header");
            first += 1;
        }

        demux.packet.type = type;
        demux.packet.pts = pts;
        demux.packet.data = .{ data = buffer.bytes + first, count = end - first };

        return *demux.packet;
    }
}

decode_pts :: (p: *u8, prefix: u8) -> float64 {
    B.assert((p[0] >> 4) == prefix && (p[0] & 1) == 1 && (p[2] & 1) == 1 && (p[4] & 1) == 1, "invalid MPEG timestamp marker");
    clock := (cast(u64) ((p[0] >> 1) & 7) << 30) | (cast(u64) p[1] << 22) | (cast(u64) (p[2] >> 1) << 15) | (cast(u64) p[3] << 7) | cast(u64) (p[4] >> 1);
    return cast(float64) clock / 90000.0;
}
