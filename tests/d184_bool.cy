// trace: D184
@config(default);
import std;

fn flip [$b: Lent.mut bool] -> void:
    change $b = not $b;
    return void;
end fn flip

fn.main [] -> void:
    let mut $on: bool = false;
    flip(Lend.mut $on);
    if $on:
        std::io::fmt_print("on{newln}");
    end if
    return void;
end fn.main
