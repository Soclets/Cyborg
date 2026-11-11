// trace: D134, D75

fn my::type [none] returns i32:
    return 1;
end fn my::type

fn.main [none] returns void:
    let $a: i32 = type();
    return void;
end fn.main
