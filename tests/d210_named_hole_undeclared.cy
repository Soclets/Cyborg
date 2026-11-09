// trace: D210
@config(default);
import std;

fn.main [] -> void:
    std::io::fmt_print("temp: {$t} C{newln}");
    return void;
end fn.main
