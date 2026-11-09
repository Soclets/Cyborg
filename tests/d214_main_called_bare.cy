// trace: D214, D75
@config(default);

fn my::main [] -> i32:
    return 7;
end fn my::main

fn.main [] -> void:
    let $n: i32 = main();
    return void;
end fn.main
