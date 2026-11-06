// trace: D111
import std;

narrowings.unit length:
    length.meters: base;
end narrowings.unit length

narrowings.unit time:
    time.seconds: base;
end narrowings.unit time

narrowings.unit speed:
    speed.meters_per_second: base;
end narrowings.unit speed

together length.meters / time.seconds = speed.meters_per_second;

fn.main [] -> void:
    let $d: i32.length.meters = 100 length.meters;
    let $t: i32.time.seconds = 8 time.seconds;
    let $v: i32.speed.meters_per_second = $d div.trunc $t;
    std::println($v as str);
    return void;
end fn.main
