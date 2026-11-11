// Unary `-` sits below `??`, so this needs `( )`, and `-1` is a literal so it doesn't.
// trace: D132, D116

fn read [none] returns i32 fails [none]:
    return 1;
end fn read

fn.main [none] returns void:
    let $x: i32 set_to 5;
    let $a: i32 set_to read() ?? -$x;
    return void;
end fn.main
