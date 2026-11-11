// trace: D214, D75

fn my::main [none] -> i32:
    return 7;
end fn my::main

fn.main [none] -> void:
    let $n: i32 = main();
    return void;
end fn.main
