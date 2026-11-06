// trace: D122
import std;

fam.unit time:
    time.seconds: base;
end fam.unit time

fn.main [] -> void:
    let $a: i64 = 9000000000;
    let $b: u8 = 200;
    let $c: i32.time.seconds = 3 time.seconds;
    let $d: i32[min: 0, max: 9] = 4;
    std::println($a as str);
    std::println($b as str);
    std::println($c as str);
    std::println($d as str);
    return void;
end fn.main
