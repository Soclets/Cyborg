// trace: D195
@config(default);
import std;

fn.main [] -> void:
    std::io::fmt_print("out{newln}");
    std::io::fmt_print.stderr("failed{newln}");
    std::io::fmt_print("out again{newln}");
    return void;
end fn.main
