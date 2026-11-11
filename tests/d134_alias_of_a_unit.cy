// trace: D134, D100
import std;
import os;

fam.unit time:
    time.seconds: base;
end fam.unit time

type.alias secs = i32[unit: time.seconds];

fn.main [none] returns void:
    let mut $out: os::stdout.unbuffered = os::stdout.unbuffered();
    let $a: secs = 3 time.seconds;
    let $b: i32[unit: time.seconds] = $a + 1 time.seconds;
    std::io::fmt_print(Lend.mut $out, "{}{newln}", $b);
    return void;
end fn.main
