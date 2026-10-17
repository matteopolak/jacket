#import "Basic";
Tls :: #import "Extensions/Tls";

main :: () {
    config: Tls.Tls_Config;
    error := Tls.tls_config_init(*config);
    assert(error.code == 0);
    Tls.tls_config_free(*config);
}
