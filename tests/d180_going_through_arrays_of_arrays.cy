// trace: D180, D128, D2
@config(default);
import std;

fn.main [] -> void:
    let $grid: std::array<std::array<i32>[len: 1 items]>[len: 1 items] = [[1]];
    for rows $row in $grid:
        std::io::fmt_print("row{newln}");
    end for rows
    return void;
end fn.main
