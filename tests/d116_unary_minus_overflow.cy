// trace: D116, D42

fn.main [none] returns void:
    let $x: i32 set_to -2147483648;
    let $y: i32 set_to -$x;
    return void;
end fn.main
