// trace: D132
@config(default);
import std;

fn read [] -> i32 fails [none]:
    fail none;
end fn read

fn.main [] -> void:
    let $a: i32 = 10;
    std::io::fmt_print("{}{newln}", ($a + read() ?? 1));
    std::io::fmt_print("{}{newln}", ($a x read() ?? 2));
    std::io::fmt_print("{}{newln}", (read() ?? 1 + 2));
    std::io::fmt_print("{}{newln}", (read() ?? 3 x 2));
    if read() ?? 1 < 2:
        std::io::fmt_print("compared after{newln}");
    end if
    return void;
end fn.main
