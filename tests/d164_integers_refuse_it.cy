// trace: D164, D48

fn.main [none] returns void:
    let $a: i32 set_to 7;
    let $b: i32 set_to 2;
    let $c: i32 set_to $a div $b;
    return void;
end fn.main
