// A function declared with `[none]` is still called with `()`.
// trace: D267
import std;

fn seven [none] -> i32:
    return 7;
end fn seven

fn.main [none] -> void:
    let $n: i32 = seven();
    std::io::fmt_print("{$n}{newln}");
    return void;
end fn.main
