// trace: D180, D128, D2
import std;
@config(default);

fn.main [] -> void:
    let $grid: std::array<std::array<i32>[len: 1 items]>[len: 1 items] = [[1]];
    for rows $row in $grid:
        std::println("row");
    end for rows
    return void;
end fn.main
