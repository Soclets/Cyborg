// `??` only uses its right side if the left failed.
// trace: D44, D2

fn read [none] returns str fails [none]:
    return "a";
end fn read

fn.main [none] returns void:
    let $s: str set_to "b";
    let $t: str set_to read() ?? Give $s;
    let $u: str set_to Give $s;
    return void;
end fn.main
