// Only one block runs, so each may give it away, and it's dead after.
// trace: D126, D2
import std;
import os;

fn eat [$s: Given str] -> void:
    let mut $out: os::stdout.unbuffered = os::stdout.unbuffered();
    std::io::fmt_print(Lend.mut $out, "{}{newln}", $s);
    return void;
end fn eat

fn pick [$s: Given str, $first: bool] -> void:
    if $first:
        eat(Give $s);
    else:
        eat(Give $s);
    end if
    return void;
end fn pick

fn.main [none] -> void:
    pick("one", 1 < 2);
    pick("two", 2 < 1);
    return void;
end fn.main
