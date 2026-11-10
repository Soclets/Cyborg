// A hole reads a value, never a call or maths.
// trace: D247
import std;

fn.main [none] -> void:
    let $count: i32 = 3;
    std::io::fmt_print("{$count + 1}{newln}");
    return void;
end fn.main
