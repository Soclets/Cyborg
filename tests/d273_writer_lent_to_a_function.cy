// A function that prints takes the writer, lent to change.
// trace: D272, D273
import std;
import os;

fn report [$out: Lent.mut os::stdout.buffered, $n: i32] -> void:
    std::io::fmt_print(Lend.mut $out, "n is {$n}{newln}");
    return void;
end fn report

fn.main [none] -> void:
    let mut $out: os::stdout.buffered = os::stdout.buffered(room: 1024 bytes);
    report(Lend.mut $out, 1);
    report(Lend.mut $out, 2);
    os::done(Give $out);
    return void;
end fn.main
