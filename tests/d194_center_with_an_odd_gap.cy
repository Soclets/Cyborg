// trace: D194
import std;
@config(default);

fn.main [] -> void:
    std::io::fmt_print("[{align: center, width: 5 cols}]{newln}", "ab");
    return void;
end fn.main
