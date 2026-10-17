#if Config.public_enabled {
    ready :: true;
}

Config :: #import,file "config.jai"(enabled = true);
