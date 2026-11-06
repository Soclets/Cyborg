// trace: D127, D75
fn true [] -> i32:
    return 1;
end fn true

fn.main [] -> void:
    let $a: i32 = true();
    return void;
end fn.main
