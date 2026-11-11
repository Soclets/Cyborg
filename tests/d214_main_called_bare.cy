// trace: D214, D75

fn my::main [none] returns i32:
    return 7;
end fn my::main

fn.main [none] returns void:
    let $n: i32 set_to main();
    return void;
end fn.main
