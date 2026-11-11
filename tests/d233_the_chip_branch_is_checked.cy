// A chip's branch is checked, though the oracle doesn't follow it.
// trace: D233
import std;
import os;

fn.main [none] returns void:
    let mut $out: os::stdout.unbuffered set_to os::stdout.unbuffered();
    let $a: i64 set_to 6;
    let $b: i64 set_to 7;
    on.target aarch64:
        let $wrong: i32 set_to $a;
    else:
        std::io::fmt_print(Lend.mut $out, "portable{newln}");
    end on.target
    return void;
end fn.main
