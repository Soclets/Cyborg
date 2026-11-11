// trace: D80, D81, D82, D92
import std;
import @csup;
import os;

fn.main [none] returns void:
    let mut $out: os::stdout.unbuffered set_to os::stdout.unbuffered();
    std::io::fmt_print(Lend.mut $out, "no manifest needed{newln}");
    return void;
end fn.main
