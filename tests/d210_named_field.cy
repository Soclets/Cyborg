// trace: D210, D247
import std;
import os;

struct reading:
    celsius: i32;
end struct reading

fn.main [none] returns void:
    let mut $out: os::stdout.unbuffered set_to os::stdout.unbuffered();
    let $r: reading set_to reading[celsius set_to 3];
    std::io::fmt_print(Lend.mut $out, "{$r.celsius}{newln}");
    return void;
end fn.main
