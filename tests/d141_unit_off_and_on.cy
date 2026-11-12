// trace: D141, D140
import std;
import os;

fam.unit time:
    time.seconds: base;
end fam.unit time

fn.main [none] -> void:
    let mut $out: os::stdout.unbuffered = os::stdout.unbuffered();
    let $xs: std::coll::array<i32>[len: 3 items] = [1, 2, 3];
    let $plain: u64 = $xs.len as u64;
    let $doubled: u64 = $plain x 2;
    std::io::fmt_print(Lend.mut $out, "{}{newln}", $doubled);
    let $back: u64[unit: items] = $doubled as u64[unit: items];
    std::io::fmt_print(Lend.mut $out, "{}{newln}", $back);
    let $t: i32[unit: time.seconds] = 3 time.seconds;
    let $n: i32 = $t as i32;
    std::io::fmt_print(Lend.mut $out, "{}{newln}", ($n x 2));
    let $f: f64.stable = 0.5;
    let $g: f64.stable[unit: time.seconds] = $f as f64.stable[unit: time.seconds];
    let $h: f64.stable = $g as f64.stable;
    if $h: f64.stable[min: 0.5, max: 0.5]:
        std::io::fmt_print(Lend.mut $out, "float too{newln}");
    end if
    return void;
end fn.main
