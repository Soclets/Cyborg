// trace: D128
import std;
@config(default);

fn.main [] -> void:
    for rows $i in range[from: 1, to: 2]:
        break rows;
    end for rows
    for rows $j in range[from: 1, to: 2]:
        std::io::fmt_print("second{newln}");
        break rows;
    end for rows
    return void;
end fn.main
