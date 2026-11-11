// trace: D42

fn.main [none] returns void:
    let $a: i32 set_to 2147483647;
    let $b: i32 set_to $a + 1;
    return void;
end fn.main
