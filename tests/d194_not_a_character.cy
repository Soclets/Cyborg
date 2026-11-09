// trace: D194
import std;
@config(default);

fn.main [] -> void:
    std::io::fmt_print("\uD800{newln}");
    return void;
end fn.main
