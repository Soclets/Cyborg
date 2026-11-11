// trace: D129, D75
import std;
import os;

fn my::finish [none] -> void:
    let mut $out: os::stdout.unbuffered = os::stdout.unbuffered();
    std::io::fmt_print(Lend.mut $out, "mine{newln}");
    return void;
end fn my::finish

fn.main [none] -> void:
    my::finish();
    return void;
end fn.main
