// `::` is for paths, and `.` after a function is a narrowing.
// trace: D196
import std;
import os;

fn.main [none] returns void:
    let mut $out: os::stdout.unbuffered = os::stdout.unbuffered();
    let $tenth: f64.stable = ~0.1;
    std::io::fmt_print::stored(Lend.mut $out, "{}{newln}", $tenth);
    return void;
end fn.main
