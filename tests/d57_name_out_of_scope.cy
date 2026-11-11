// trace: D57, D33

fn.main [none] returns void:
    if 1 below 2:
        let $a: i32 set_to 1;
    end if
    let $b: i32 set_to $a;
    return void;
end fn.main
