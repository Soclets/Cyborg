// A block's names end with it, so the next block can use the name again.
// trace: D57, D33
@config(default);
import std;

fn.main [] -> void:
    if 1 < 2:
        let $a: i32 = 1;
        std::io::fmt_print("{}{newln}", $a);
    end if
    if 1 < 2:
        let $a: i32 = 2;
        std::io::fmt_print("{}{newln}", $a);
    end if
    return void;
end fn.main
