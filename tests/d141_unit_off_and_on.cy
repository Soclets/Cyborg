// trace: D141, D140
import std;

fam.unit time:
    time.seconds: base;
end fam.unit time

fn.main [] -> void:
    let $xs: std::array<i32>[len: 3 items] = [1, 2, 3];
    let $plain: u64 = $xs.len as u64;
    let $doubled: u64 = $plain x 2;
    std::println($doubled as str);
    let $back: u64.items = $doubled as u64.items;
    std::println($back as str);
    let $t: i32.time.seconds = 3 time.seconds;
    let $n: i32 = $t as i32;
    std::println(($n x 2) as str);
    let $f: f64 = 0.5;
    let $g: f64.time.seconds = $f as f64.time.seconds;
    if $g == 0.5 time.seconds:
        std::println("float too");
    end if
    return void;
end fn.main
