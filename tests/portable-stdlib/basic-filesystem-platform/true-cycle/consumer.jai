Config :: #import,file "config.jai"(enabled = true);

#if Config.ready {
    api :: (value: int) -> int {
        return value + 1;
    }
}
