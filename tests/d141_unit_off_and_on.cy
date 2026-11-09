// trace: D141, D140
@config(default);
import std;

fam.unit time:
    time.seconds: base;
end fam.unit time

fn.main [] -> void:
    let $xs: std::array<i32>[len: 3 items] = [1, 2, 3];
    let $plain: u64 = $xs.len as u64;
    let $doubled: u64 = $plain x 2;
    std::io::fmt_print("{}{newln}", $doubled);
    let $back: u64.items = $doubled as u64.items;
    std::io::fmt_print("{}{newln}", $back);
    let $t: i32.time.seconds = 3 time.seconds;
    let $n: i32 = $t as i32;
    std::io::fmt_print("{}{newln}", ($n x 2));
    let $f: f64 = 0.5;
    let $g: f64.time.seconds = $f as f64.time.seconds;
    let $h: f64 = $g as f64;
    if $h: f64[min: 0.5, max: 0.5]:
        std::io::fmt_print("float too{newln}");
    end if
    return void;
end fn.main
