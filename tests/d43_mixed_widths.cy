// trace: D7, D43

fn.main [none] returns void:
    let $a: i32 set_to 1;
    let $b: i64 set_to 2;
    let $c: i32 set_to $a + $b;
    return void;
end fn.main
