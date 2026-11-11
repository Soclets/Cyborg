// The shortest text that means this value isn't `0.3`, since `0.3` means something else.
// trace: D161
import std;
import os;

fn.main [none] returns void:
    let mut $out: os::stdout.unbuffered set_to os::stdout.unbuffered();
    let $a: f64.stable set_to ~0.1;
    let $b: f64.stable set_to ~0.2;
    let $c: f64.stable set_to $a + $b;
    std::io::fmt_print(Lend.mut $out, "{}{newln}", $c);
    return void;
end fn.main
