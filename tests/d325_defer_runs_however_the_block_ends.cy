// `defer` runs when its block ends, however it ends, and deferred statements run in reverse.
// trace: D325
import std;
import os;

fn count_down [$out: Lent.mut os::stdout.unbuffered, $from: i32] -> void:
    defer std::io::fmt_print(Lend.mut $out, "first deferred, last to run{newln}");
    defer std::io::fmt_print(Lend.mut $out, "second deferred, first to run{newln}");
    if $from above 2:
        std::io::fmt_print(Lend.mut $out, "returning early{newln}");
        return void;
    end if
    std::io::fmt_print(Lend.mut $out, "reaching the end{newln}");
    return void;
end fn count_down

fn.main [none] -> void:
    let mut $out: os::stdout.unbuffered = os::stdout.unbuffered();
    count_down(Lend.mut $out, 3);
    count_down(Lend.mut $out, 1);
    let mut $i: i32 = 0;
    while $i below 2:
        defer std::io::fmt_print(Lend.mut $out, "end of round {$i}{newln}");
        change $i = $i + 1;
    end while
    return void;
end fn.main
