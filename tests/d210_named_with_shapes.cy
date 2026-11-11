// trace: D210, D247
import std;
import os;

fn.main [none] returns void:
    let mut $out: os::stdout.unbuffered = os::stdout.unbuffered();
    let $t: f64.stable = 2.5;
    std::io::fmt_print(Lend.mut $out, "{$t, places: 2}{newln}");
    return void;
end fn.main
