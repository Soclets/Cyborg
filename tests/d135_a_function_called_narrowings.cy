// `narrowings` isn't a word of Cyborg now, so it's free as a name.
// trace: D135, D75
import std;
import os;

fn narrowings [none] -> i32:
    return 1;
end fn narrowings

fn.main [none] -> void:
    let mut $out: os::stdout.unbuffered = os::stdout.unbuffered();
    std::io::fmt_print(Lend.mut $out, "{}{newln}", narrowings());
    return void;
end fn.main
