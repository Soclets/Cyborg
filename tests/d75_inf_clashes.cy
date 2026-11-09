// trace: D75
@config(default);

fn my::inf [] -> i32:
    return 1;
end fn my::inf

fn.main [] -> void:
    let $a: i32 = inf();
    return void;
end fn.main
