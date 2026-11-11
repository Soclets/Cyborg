// `on.target`'s narrowings, like `.os`, aren't designed.
// trace: D233
import std;
import os;

fn.main [none] -> void:
    let mut $out: os::stdout.unbuffered = os::stdout.unbuffered();
    let $a: i64 = 6;
    let $b: i64 = 7;
    on.target.os linux:
        std::io::fmt_print(Lend.mut $out, "linux{newln}");
    end on.target
    return void;
end fn.main
