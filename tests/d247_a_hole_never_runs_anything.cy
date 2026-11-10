// A hole reads a value, never a call or maths.
// trace: D247
@config(default);
import std;

fn.main [] -> void:
    let $count: i32 = 3;
    std::io::fmt_print("{$count + 1}{newln}");
    return void;
end fn.main
