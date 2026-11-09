// trace: D134, D75
@config(default);

fn my::type [] -> i32:
    return 1;
end fn my::type

fn.main [] -> void:
    let $a: i32 = type();
    return void;
end fn.main
