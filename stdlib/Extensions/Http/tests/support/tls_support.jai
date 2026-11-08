// Client configuration cleanup for HTTP fixtures running on a scheduler.
Tls :: #import "Extensions/Tls";

// Close every idle client connection before releasing its borrowed TLS configuration.
// Responses must already be closed. Example: defer free_client_config(*config);
free_client_config :: (config: *Tls.Tls_Config) {
    close_idle_connections();

    statistics := http_statistics();

    assert(statistics.idle_connections == 0, "client TLS configurations must outlive pooled connections");
    assert(statistics.open_responses == 0, "close responses before freeing client TLS configurations");
    Tls.tls_config_free(config);
}
