// trace: D128
import std;
import os;

fn.main [none] -> void:
    let mut $out: os::stdout.unbuffered = os::stdout.unbuffered();
    for rows $i in range[from: 1, to: 2]:
        break rows;
    end for rows
    for rows $j in range[from: 1, to: 2]:
        std::io::fmt_print(Lend.mut $out, "second{newln}");
        break rows;
    end for rows
    return void;
end fn.main
