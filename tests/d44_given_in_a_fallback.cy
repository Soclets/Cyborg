// `??` only uses its right side if the left failed.
// trace: D44, D2
fn read [] -> str:
    fails [none];

    return "a";
end fn read

fn.main [] -> void:
    let $s: str = "b";
    let $t: str = read() ?? Give $s;
    let $u: str = Give $s;
    return void;
end fn.main
