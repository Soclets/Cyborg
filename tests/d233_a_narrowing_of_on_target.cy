// `on.target`'s narrowings, like `.os`, aren't designed.
// trace: D233
import std;
import os;

fn.main [none] returns void:
    let mut $out: os::stdout.unbuffered set_to os::stdout.unbuffered();
    let $a: i64 set_to 6;
    let $b: i64 set_to 7;
    on.target.os linux:
        std::io::fmt_print(Lend.mut $out, "linux{newln}");
    end on.target
    return void;
end fn.main
