// trace: D98
@config(default);
import std;

fn.main [] -> void:
    std::io::fmt_print.buffered("hi{newln}");
    return void;
end fn.main
