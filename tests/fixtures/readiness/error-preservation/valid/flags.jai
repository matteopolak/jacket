#if Config.enabled {
    ready :: true;
}

Config :: #import,file "config.jai"(enabled = true);
