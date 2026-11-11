// trace: D184
import std;
import os;

fn flip [$b: Lent.mut bool] -> void:
    change $b = not $b;
    return void;
end fn flip

fn.main [none] -> void:
    let mut $out: os::stdout.unbuffered = os::stdout.unbuffered();
    let mut $on: bool = false;
    flip(Lend.mut $on);
    if $on:
        std::io::fmt_print(Lend.mut $out, "on{newln}");
    end if
    return void;
end fn.main
