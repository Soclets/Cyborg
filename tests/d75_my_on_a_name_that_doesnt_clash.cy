// trace: D75
fn total [] -> i32:
    return 1;
end fn total

fn.main [] -> void:
    let $a: i32 = my::total();
    return void;
end fn.main
