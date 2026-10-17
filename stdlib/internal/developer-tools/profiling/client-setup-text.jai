// Independently authored client setup; purpose, no literal-text compatibility promise.

#if !MANUAL_MODE && !USE_GRAPHICS && !DO_HISTORY {
    TO_INSERT :: #string PROFILE_CLIENT_SETUP
#scope_file
Independent_Profile_Client :: #import "Iprof"(IMPORT_MODE = .CLIENT, MANUAL_MODE = false, USE_GRAPHICS = false, DO_HISTORY = false);
#add_context _Iprof :: Independent_Profile_Client;
PROFILE_CLIENT_SETUP
}

#if !MANUAL_MODE && !USE_GRAPHICS && DO_HISTORY {
    TO_INSERT :: #string PROFILE_CLIENT_SETUP
#scope_file
Independent_Profile_Client :: #import "Iprof"(IMPORT_MODE = .CLIENT, MANUAL_MODE = false, USE_GRAPHICS = false, DO_HISTORY = true);
#add_context _Iprof :: Independent_Profile_Client;
PROFILE_CLIENT_SETUP
}

#if !MANUAL_MODE && USE_GRAPHICS && !DO_HISTORY {
    TO_INSERT :: #string PROFILE_CLIENT_SETUP
#scope_file
Independent_Profile_Client :: #import "Iprof"(IMPORT_MODE = .CLIENT, MANUAL_MODE = false, USE_GRAPHICS = true, DO_HISTORY = false);
#add_context _Iprof :: Independent_Profile_Client;
PROFILE_CLIENT_SETUP
}

#if !MANUAL_MODE && USE_GRAPHICS && DO_HISTORY {
    TO_INSERT :: #string PROFILE_CLIENT_SETUP
#scope_file
Independent_Profile_Client :: #import "Iprof"(IMPORT_MODE = .CLIENT, MANUAL_MODE = false, USE_GRAPHICS = true, DO_HISTORY = true);
#add_context _Iprof :: Independent_Profile_Client;
PROFILE_CLIENT_SETUP
}

#if MANUAL_MODE && !USE_GRAPHICS && !DO_HISTORY {
    TO_INSERT :: #string PROFILE_CLIENT_SETUP
#scope_file
Independent_Profile_Client :: #import "Iprof"(IMPORT_MODE = .CLIENT, MANUAL_MODE = true, USE_GRAPHICS = false, DO_HISTORY = false);
#add_context _Iprof :: Independent_Profile_Client;
PROFILE_CLIENT_SETUP
}

#if MANUAL_MODE && !USE_GRAPHICS && DO_HISTORY {
    TO_INSERT :: #string PROFILE_CLIENT_SETUP
#scope_file
Independent_Profile_Client :: #import "Iprof"(IMPORT_MODE = .CLIENT, MANUAL_MODE = true, USE_GRAPHICS = false, DO_HISTORY = true);
#add_context _Iprof :: Independent_Profile_Client;
PROFILE_CLIENT_SETUP
}

#if MANUAL_MODE && USE_GRAPHICS && !DO_HISTORY {
    TO_INSERT :: #string PROFILE_CLIENT_SETUP
#scope_file
Independent_Profile_Client :: #import "Iprof"(IMPORT_MODE = .CLIENT, MANUAL_MODE = true, USE_GRAPHICS = true, DO_HISTORY = false);
#add_context _Iprof :: Independent_Profile_Client;
PROFILE_CLIENT_SETUP
}

#if MANUAL_MODE && USE_GRAPHICS && DO_HISTORY {
    TO_INSERT :: #string PROFILE_CLIENT_SETUP
#scope_file
Independent_Profile_Client :: #import "Iprof"(IMPORT_MODE = .CLIENT, MANUAL_MODE = true, USE_GRAPHICS = true, DO_HISTORY = true);
#add_context _Iprof :: Independent_Profile_Client;
PROFILE_CLIENT_SETUP
}
