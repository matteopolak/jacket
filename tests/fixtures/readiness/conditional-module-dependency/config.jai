#module_parameters(enabled: bool);

ready :: enabled;

#if Peer.api(41) == 42 {
    extra :: true;
}

Peer :: #import,file "consumer.jai";
