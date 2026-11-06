// trace: D109, D112
import std;

narrowings.unit time:
    time.minutes = 60 std::time.seconds;
end narrowings.unit time

fn.main [] -> void:
    let $m: i32.time.minutes = 2 time.minutes;
    std::println($m as str);
    return void;
end fn.main
