// A failed check flushes every buffered writer still holding output before its message, so
// the last thing printed before the crash shows.
// trace: D274, D273
import std;
import os;

fn.main [none] returns void:
    let mut $out: os::stdout.buffered set_to os::stdout.buffered(size: 1024 bytes);
    let $big: i32 set_to 2147483647;
    std::io::fmt_print(Lend.mut $out, "before{newln}");
    let $over: i32 set_to $big + 1;
    std::io::fmt_print(Lend.mut $out, "{$over}{newln}");
    os::done(Give $out);
    return void;
end fn.main
