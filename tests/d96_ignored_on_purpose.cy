// trace: D96
import std;
@config(default);

fn.main [] -> void:
    std::io::fmt_print("hi{newln}") ?? void;
    return void;
end fn.main
