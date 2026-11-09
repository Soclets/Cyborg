// trace: D96
@config(default);
import std;

fn.main [] -> void:
    std::io::fmt_print("hi{newln}") ?? void;
    return void;
end fn.main
