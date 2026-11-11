// A real program, now that lengths can reach positions: print a list backwards, by position.
// trace: D126, D128, D133, D140, D141, D142
import std;
import os;

fn print_backwards [$xs: std::list<str>[len.max: 8 items]] returns void:
    let mut $out: os::stdout.unbuffered set_to os::stdout.unbuffered();
    let mut $at: u64 set_to $xs.len as u64;
    while $at above 0:
        std::io::fmt_print(Lend.mut $out, "{}{newln}", $xs[pos: $at]);
        change $at set_to $at - 1;
    end while
    return void;
end fn print_backwards

fn.main [none] returns void:
    let mut $lines: std::list<str>[len.max: 8 items] set_to empty;
    $lines.std::%%append(val: "first");
    $lines.std::%%append(val: "second");
    $lines.std::%%append(val: "third");
    print_backwards(Lend $lines);
    return void;
end fn.main
