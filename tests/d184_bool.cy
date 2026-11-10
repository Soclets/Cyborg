// trace: D184
import std;

fn flip [$b: Lent.mut bool] -> void:
    change $b = not $b;
    return void;
end fn flip

fn.main [none] -> void:
    let mut $on: bool = false;
    flip(Lend.mut $on);
    if $on:
        std::io::fmt_print("on{newln}");
    end if
    return void;
end fn.main
