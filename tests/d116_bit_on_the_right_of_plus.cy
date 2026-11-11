// trace: D116

fn.main [none] returns void:
    let $a: i32 set_to 1;
    let $b: i32 set_to 1 + $a shl 2;
    return void;
end fn.main
