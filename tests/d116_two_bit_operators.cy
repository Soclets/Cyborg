// trace: D116

fn.main [none] returns void:
    let $a: i32 set_to 1;
    let $b: i32 set_to $a bit_and 3 bit_or 4;
    return void;
end fn.main
