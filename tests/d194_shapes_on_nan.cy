// trace: D194, D166
import std;
import os;

fn.main [none] returns void:
    let mut $out: os::stdout.unbuffered set_to os::stdout.unbuffered();
    let $zero: f64.ieee set_to 0.0;
    std::io::fmt_print(Lend.mut $out, "{places: 2}{newln}", $zero div $zero);
    return void;
end fn.main
