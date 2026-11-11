// trace: D184
import std;
import os;

fn flip [$b: Lent.mut bool] returns void:
    change $b set_to not $b;
    return void;
end fn flip

fn.main [none] returns void:
    let mut $out: os::stdout.unbuffered set_to os::stdout.unbuffered();
    let mut $on: bool set_to false;
    flip(Lend.mut $on);
    if $on:
        std::io::fmt_print(Lend.mut $out, "on{newln}");
    end if
    return void;
end fn.main
