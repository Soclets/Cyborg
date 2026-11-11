// Unary `-` sits below `??`, so this needs `( )`, and `-1` is a literal so it doesn't.
// trace: D132, D116

fn read [none] -> i32 fails [none]:
    return 1;
end fn read

fn.main [none] -> void:
    let $x: i32 = 5;
    let $a: i32 = read() ?? -$x;
    return void;
end fn.main
