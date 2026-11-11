// trace: D180, D128, D2
import std;
import os;

fn.main [none] -> void:
    let mut $out: os::stdout.unbuffered = os::stdout.unbuffered();
    let $grid: std::array<std::array<i32>[len: 1 items]>[len: 1 items] = [[1]];
    for rows $row in $grid:
        std::io::fmt_print(Lend.mut $out, "row{newln}");
    end for rows
    return void;
end fn.main
