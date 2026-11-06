// trace: D75
fn inf [] -> i32:
    return 1;
end fn inf

fn.main [] -> void:
    let $a: i32 = inf();
    return void;
end fn.main
