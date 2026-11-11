// trace: D42

fn.main [none] returns void:
    let $a: i32 set_to 65536;
    let $b: i32 set_to $a x $a;
    return void;
end fn.main
