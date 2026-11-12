// trace: D133
import std;
import os;

fn.main [none] -> void:
    let mut $out: os::stdout.unbuffered = os::stdout.unbuffered();
    let mut $grid: std::coll::array<std::coll::array<i32>[len: 2 items]>[len: 2 items] = [[1, 2], [3, 4]];
    std::io::fmt_print(Lend.mut $out, "{}{newln}", $grid[pos: 2][pos: 1]);
    change $grid[pos: 1][offset.items: 1] = 8;
    std::io::fmt_print(Lend.mut $out, "{}{newln}", $grid[pos: 1][pos: 2]);
    std::io::fmt_print(Lend.mut $out, "{}{newln}", $grid[pos: 1].len);
    return void;
end fn.main
