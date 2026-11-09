// trace: D214
@config(default);
import std;

fn my::main [] -> i32:
    return 7;
end fn my::main

fn.main [] -> void:
    std::io::fmt_print("{}{newln}", my::main());
    return void;
end fn.main
