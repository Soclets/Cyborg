// trace: D126, D33

fn.main [none] returns void:
    if 1 below 2:
        let $a: i32 set_to 1;
    else:
        let $b: i32 set_to $a;
    end if
    return void;
end fn.main
