// `.stored` is about floats, so an integer prints as it always does.
// trace: D194, D162
import std;

fn.main [none] -> void:
    let $n: i32 = 5;
    std::io::fmt_print.stored("{}{newln}", $n);
    return void;
end fn.main
