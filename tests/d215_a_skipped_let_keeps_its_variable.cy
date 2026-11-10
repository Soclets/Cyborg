// A skipped let still creates its variable, with no value.
// trace: D215, D203
import std;

fn.main [none] -> void:
    let $sum: i32 = totl(5);
    std::io::fmt_print("{}{newln}", $sum);
    return void;
end fn.main
