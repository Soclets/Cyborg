// By D116's table this is `($a + read()) ?? 0`, and what `??` catches inside a sum isn't stated.
// trace: D116, D44
fn read [] -> i32:
    fails [none];

    return 1;
end fn read

fn.main [] -> void:
    let $a: i32 = 1;
    let $b: i32 = $a + read() ?? 0;
    return void;
end fn.main
