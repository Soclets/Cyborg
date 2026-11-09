// trace: D126, D11
@config(default);
import std;

fn.main [] -> void:
    if 1 < 2:
        std::io::fmt_print("yes{newln}");
        return void;
    else:
        return void;
    end if
end fn.main
