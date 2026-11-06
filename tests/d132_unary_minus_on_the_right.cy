// Unary `-` sits below `??`, so this needs `( )`, and `-1` is a literal so it doesn't.
// trace: D132, D116
fn read [] -> i32:
    fails [none];

    return 1;
end fn read

fn.main [] -> void:
    let $x: i32 = 5;
    let $a: i32 = read() ?? -$x;
    return void;
end fn.main
