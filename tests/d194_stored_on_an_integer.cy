// `.stored` is about floats, so an integer prints as it always does.
// trace: D194, D162
import std;
import os;

fn.main [none] returns void:
    let mut $out: os::stdout.unbuffered set_to os::stdout.unbuffered();
    let $n: i32 set_to 5;
    std::io::fmt_print.stored(Lend.mut $out, "{}{newln}", $n);
    return void;
end fn.main
