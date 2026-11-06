// trace: D133
import std;

fn.main [] -> void:
    let mut $grid: std::array<std::array<i32>[len: 2 items]>[len: 2 items] = [[1, 2], [3, 4]];
    std::println($grid[pos: 2][pos: 1] as str);
    change $grid[pos: 1][offset: 1] = 8;
    std::println($grid[pos: 1][pos: 2] as str);
    std::println($grid[pos: 1].len as str);
    return void;
end fn.main
