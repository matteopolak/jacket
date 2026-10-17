#module_parameters(FLAG := true, SECOND := true);

Token :: struct {
    marker: u64;
}

token: Token;
state: int;
both :: FLAG && SECOND;
