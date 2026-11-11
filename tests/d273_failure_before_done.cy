// What a buffered writer's output does when a failure stops the program first isn't decided.
// trace: D273
import std;
import os;

fn.main [none] -> void:
    let mut $out: os::stdout.buffered = os::stdout.buffered(size: 1024 bytes);
    let $big: i32 = 2147483647;
    std::io::fmt_print(Lend.mut $out, "before{newln}");
    let $over: i32 = $big + 1;
    std::io::fmt_print(Lend.mut $out, "{$over}{newln}");
    os::done(Give $out);
    return void;
end fn.main
