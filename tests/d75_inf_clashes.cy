// trace: D75

fn my::inf [none] -> i32:
    return 1;
end fn my::inf

fn.main [none] -> void:
    let $a: i32 = inf();
    return void;
end fn.main
