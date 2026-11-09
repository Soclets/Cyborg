// trace: D134, D100
import std;
@config(default);

fam.unit time:
    time.seconds: base;
end fam.unit time

type.alias secs = i32.time.seconds;

fn.main [] -> void:
    let $a: secs = 3 time.seconds;
    let $b: i32.time.seconds = $a + 1 time.seconds;
    std::io::fmt_print("{}{newln}", $b);
    return void;
end fn.main
