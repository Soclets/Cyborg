// trace: D75

fn total [none] returns i32:
    return 1;
end fn total

fn.main [none] returns void:
    let $a: i32 set_to my::total();
    return void;
end fn.main
