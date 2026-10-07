QUERY :: #string SQL
SELECT name FROM users; /* left open
SQL;

SCRIPT :: #string py
def twice(x): return x * 2
py;

SHADER :: #string,cr WGSL
@vertex fn vs() -> @builtin(position) vec4f { return vec4f(0.0); }
    WGSL;

TEXT :: #string END
plain text
END;

after := 1;
