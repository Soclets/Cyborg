// trace: D169
import std;
import os;

struct reading:
    celsius: i32;
    time: i32;
end struct reading

fn.main [none] returns void:
    let mut $out: os::stdout.unbuffered set_to os::stdout.unbuffered();
    let mut $r: reading set_to reading[celsius set_to 21, time set_to 3600];
    let $view: Lend from $r reading set_to Lend $r;
    std::io::fmt_print(Lend.mut $out, "{}{newln}", $view.celsius);
    change $r.celsius set_to 22;
    std::io::fmt_print(Lend.mut $out, "{}{newln}", $r.celsius);
    return void;
end fn.main
