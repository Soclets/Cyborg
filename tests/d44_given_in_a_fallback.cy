// `??` only uses its right side if the left failed.
// trace: D44, D2

fn read [none] -> str[len.max: 32 bytes] fails [none]:
    return "a";
end fn read

fn.main [none] -> void:
    let $s: str[len.max: 32 bytes] = "b";
    let $t: str[len.max: 32 bytes] = read() ?? Give $s;
    let $u: str[len.max: 32 bytes] = Give $s;
    return void;
end fn.main
