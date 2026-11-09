// trace: D194
@config(default);
import std;

fn.main [] -> void:
    std::io::fmt_print("[{align: center, width: 5 cols}]{newln}", "ab");
    return void;
end fn.main
