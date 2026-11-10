// trace: D214
import std;

fn my::main [none] -> i32:
    return 7;
end fn my::main

fn.main [none] -> void:
    std::io::fmt_print("{}{newln}", my::main());
    return void;
end fn.main
