// trace: D129, D75
import std;
import os;

fn my::finish [none] returns void:
    let mut $out: os::stdout.unbuffered set_to os::stdout.unbuffered();
    std::io::fmt_print(Lend.mut $out, "mine{newln}");
    return void;
end fn my::finish

fn.main [none] returns void:
    my::finish();
    return void;
end fn.main
