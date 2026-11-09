// A real program, now that lengths can reach positions: print a list backwards, by position.
// trace: D126, D128, D133, D140, D141, D142
@config(default);
import std;

fn print_backwards [$xs: std::list<str>[len.max: 8 items]] -> void:
    let mut $at: u64 = $xs.len as u64;
    while $at > 0:
        std::io::fmt_print("{}{newln}", $xs[pos: $at]);
        change $at = $at - 1;
    end while
    return void;
end fn print_backwards

fn.main [] -> void:
    let mut $lines: std::list<str>[len.max: 8 items] = empty;
    $lines.std::%%append(val: "first");
    $lines.std::%%append(val: "second");
    $lines.std::%%append(val: "third");
    print_backwards(Lend $lines);
    return void;
end fn.main
