// `together` lines are gone, replaced by a unit declared once with `per` and `x`.
// trace: D111, D242
import std;
import os;

fam.unit length:
    length.meters: base;
end fam.unit length

fam.unit time:
    time.seconds: base;
end fam.unit time

fam.unit speed:
    speed.meters_per_second: base;
end fam.unit speed

together length.meters / time.seconds = speed.meters_per_second;

fn.main [none] returns void:
    let mut $out: os::stdout.unbuffered = os::stdout.unbuffered();
    let $d: i32[unit: length.meters] = 100 length.meters;
    let $t: i32[unit: time.seconds] = 8 time.seconds;
    let $v: i32[unit: speed.meters_per_second] = $d div.trunc $t;
    std::io::fmt_print(Lend.mut $out, "{}{newln}", $v);
    return void;
end fn.main
