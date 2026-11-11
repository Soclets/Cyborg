// An unhandled failure flushes every buffered writer too, stdout and stderr alike, in the order
// they were made.
// trace: D274, D96
import std;
import os;

fn half [$n: i32] returns i32 fails [odd]:
    if $n mod.trunc 2 != 0:
        fail odd;
    end if
    return $n div.trunc 2;
end fn half

fn.main [none] returns void:
    let mut $out: os::stdout.buffered set_to os::stdout.buffered(size: 1024 bytes);
    let mut $err: os::stderr.buffered set_to os::stderr.buffered(size: 1024 bytes);
    std::io::fmt_print(Lend.mut $out, "{}{newln}", half(4));
    std::io::fmt_print(Lend.mut $err, "about to halve 3{newln}");
    std::io::fmt_print(Lend.mut $out, "{}{newln}", half(3));
    os::done(Give $out);
    os::done(Give $err);
    return void;
end fn.main
