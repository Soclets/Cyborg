// trace: D161, D122
import std;
import os;

fam.unit time:
    time.seconds: base;
end fam.unit time

fn.main [none] returns void:
    let mut $out: os::stdout.unbuffered = os::stdout.unbuffered();
    let $t: f64.stable[unit: time.seconds] = 1.5 time.seconds;
    std::io::fmt_print(Lend.mut $out, "{}{newln}", $t);
    return void;
end fn.main
