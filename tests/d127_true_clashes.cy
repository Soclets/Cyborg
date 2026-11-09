// trace: D127, D75
@config(default);

fn my::true [] -> i32:
    return 1;
end fn my::true

fn.main [] -> void:
    let $a: i32 = true();
    return void;
end fn.main
