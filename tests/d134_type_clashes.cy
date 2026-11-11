// trace: D134, D75

fn my::type [none] -> i32:
    return 1;
end fn my::type

fn.main [none] -> void:
    let $a: i32 = type();
    return void;
end fn.main
