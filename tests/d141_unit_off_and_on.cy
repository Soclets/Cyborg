// trace: D141, D140
import std;
import os;

fam.unit time:
    time.seconds: base;
end fam.unit time

fn.main [none] returns void:
    let mut $out: os::stdout.unbuffered set_to os::stdout.unbuffered();
    let $xs: std::array<i32>[len: 3 items] set_to [1, 2, 3];
    let $plain: u64 set_to $xs.len as u64;
    let $doubled: u64 set_to $plain x 2;
    std::io::fmt_print(Lend.mut $out, "{}{newln}", $doubled);
    let $back: u64[unit: items] set_to $doubled as u64[unit: items];
    std::io::fmt_print(Lend.mut $out, "{}{newln}", $back);
    let $t: i32[unit: time.seconds] set_to 3 time.seconds;
    let $n: i32 set_to $t as i32;
    std::io::fmt_print(Lend.mut $out, "{}{newln}", ($n x 2));
    let $f: f64.stable set_to 0.5;
    let $g: f64.stable[unit: time.seconds] set_to $f as f64.stable[unit: time.seconds];
    let $h: f64.stable set_to $g as f64.stable;
    if $h: f64.stable[min: 0.5, max: 0.5]:
        std::io::fmt_print(Lend.mut $out, "float too{newln}");
    end if
    return void;
end fn.main
