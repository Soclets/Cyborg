// trace: D128, D75
import std;
@config(default);

fn.main [] -> void:
    for x $i in range[from: 1, to: 2]:
        std::io::fmt_print("{}{newln}", $i);
        break my::x;
    end for x
    return void;
end fn.main
