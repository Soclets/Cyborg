// 4.8's example, with lists, since going through arrays of arrays isn't decided.
// trace: D128
import std;
import os;

fn.main [none] -> void:
    let mut $out: os::stdout.unbuffered = os::stdout.unbuffered();
    let $grid: std::list<std::list<i32>[len.max: 2 items]>[len.max: 2 items] = [[1, 2], [3, 4]];
    let $target: i32 = 3;
    for rows $row in $grid:
        for $cell in $row:
            if $cell == $target:
                std::io::fmt_print(Lend.mut $out, "found{newln}");
                break rows;
            end if
            std::io::fmt_print(Lend.mut $out, "{}{newln}", $cell);
        end for
    end for rows
    return void;
end fn.main
