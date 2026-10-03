# Icon

## Public declaration inventory

Visible public declarations and module-conditional variants are listed below. Procedure bodies and aggregate initializers are omitted; complete public type members are listed separately.

- `get_plugin :: () -> *Metaprogram_Plugin`
- `message :: (p: *Metaprogram_Plugin, message: *Message)`
- `shutdown :: (p: *Metaprogram_Plugin)`
- `init :: (p: *Metaprogram_Plugin, options: [] string) -> bool`
- `log_help :: (p: *Metaprogram_Plugin)`
- `HELP_STRING :: #string DONE`

## Behavioral contract and constraints

`get_plugin` allocates a `My_Plugin` record and fills the public plugin callbacks. `init` accepts `-icon <filename>`; no other option is accepted. A missing value or unknown option logs an error and returns false. Repeated `-icon` options replace the prior stored view, so the last one is used. The filename is stored as a string view from the options and must remain valid through the post-write message. `shutdown` frees the plugin record. No additional plugin-owned filename buffer is allocated.

The plugin does not use `communication_proc` or any key/value message protocol. Its `message` callback responds only to a `PHASE` message whose phase is `POST_WRITE_EXECUTABLE`; it builds the executable path from that message's workspace output path and executable name. With no `-icon` option it logs and leaves the executable unchanged. On Windows it passes the ICO filename to `Windows_Resources.set_icon_by_filename` using that API's default resource ID and default preserve-existing-resources setting; failure is logged. On other target operating systems it logs that icon setting is unsupported. It does not convert PNG/JPEG/BMP input: ICO is the supported input format.

The host is responsible for calling the plugin callbacks and for the option storage lifetime. If an implementation invokes `message` without a usable workspace/output path or executable name, this module defines no fallback path policy. Platform/resource support depends on the selected executable target; actual Windows resource mutation needs Windows verification.
