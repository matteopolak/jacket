// Curl: bindings for the system libcurl, the C library for URL transfers (HTTP, SMTP, FTP and many more).
//
// This module is the C API as it stands: `curl_easy_init`, `curl_easy_setopt`, `curl_easy_perform` and the rest keep their
// C names, arguments and results, with `CURLcode` and the option enums as Jai enums. Nothing is wrapped, so the C rules
// apply: a handle from `curl_easy_init` is released with `curl_easy_cleanup`, strings handed to libcurl must be
// NUL-terminated (`temp_c_string`), `curl_global_init` runs once per program before the first handle is made, and
// every transfer function blocks until the transfer is over unless the multi interface is used. Failure is a `CURLcode`
// other than `.OK`, and `curl_easy_strerror` turns it into text. The libcurl shared library has to be installed on the
// machine that runs the program; it is not part of the toolchain. `Mail` is built on this module, and `Extensions/Http`
// is the HTTP client that needs no C library.
//
// ```jai,no_run
// curl_global_init(0);
//
// easy := curl_easy_init();
// assert(easy != null);
// defer curl_easy_cleanup(easy);
//
// curl_easy_setopt(easy, .URL, temp_c_string("https://example.com/"));
// code := curl_easy_perform(easy);
//
// if code != .OK print("transfer failed: %\n", to_string(curl_easy_strerror(code)));
// ```
