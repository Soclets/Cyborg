// trace: D128
import std;
@config(default);

fn.main [] -> void:
    let mut $n: i32 = 3;
    while $n > 0:
        std::io::fmt_print("{}{newln}", $n);
        change $n = $n - 1;
    end while
    while false:
        std::io::fmt_print("never{newln}");
    end while
    return void;
end fn.main
