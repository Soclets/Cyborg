// trace: D198, D199
import std::io::fmt_print;
@config(default);

fn.main [] -> void:
    std::io::fmt_print("only this{newln}");
    std::io::fmt_print.stderr("and its narrowings{newln}");
    return void;
end fn.main
