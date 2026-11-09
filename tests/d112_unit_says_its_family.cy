// trace: D99, D112, D109
import std;
@config(default);

fam.unit time:
    time.seconds: base;
    time.minutes = 60 time.seconds;
end fam.unit time

fn.main [] -> void:
    let $a: i32.time.seconds = 3600 time.seconds;
    let $b: i32.time.seconds = 5 time.seconds;
    let $c: i32.time.seconds = $a + $b;
    std::println($c as str);
    if $c > $a:
        std::println("more");
    end if
    return void;
end fn.main
