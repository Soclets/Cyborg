// trace: D194
@config(default);
import std;

fn.main [] -> void:
    std::io::fmt_print("\uD800{newln}");
    return void;
end fn.main
