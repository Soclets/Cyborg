// trace: D184
import std;
@config(default);

fn flip [$b: Lent.mut bool] -> void:
    change $b = not $b;
    return void;
end fn flip

fn.main [] -> void:
    let mut $on: bool = false;
    flip(Lend.mut $on);
    if $on:
        std::println("on");
    end if
    return void;
end fn.main
