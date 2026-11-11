// A function that prints takes the writer, lent to change.
// trace: D272, D273
import std;
import os;

fn report [$out: Lent.mut os::stdout.buffered, $n: i32] returns void:
    std::io::fmt_print(Lend.mut $out, "n is {$n}{newln}");
    return void;
end fn report

fn.main [none] returns void:
    let mut $out: os::stdout.buffered set_to os::stdout.buffered(size: 1024 bytes);
    report(Lend.mut $out, 1);
    report(Lend.mut $out, 2);
    os::done(Give $out);
    return void;
end fn.main
