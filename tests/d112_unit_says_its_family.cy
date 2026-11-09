// trace: D99, D112, D109
@config(default);
import std;

fam.unit time:
    time.seconds: base;
    time.minutes = 60 time.seconds;
end fam.unit time

fn.main [] -> void:
    let $a: i32.time.seconds = 3600 time.seconds;
    let $b: i32.time.seconds = 5 time.seconds;
    let $c: i32.time.seconds = $a + $b;
    std::io::fmt_print("{}{newln}", $c);
    if $c > $a:
        std::io::fmt_print("more{newln}");
    end if
    return void;
end fn.main
