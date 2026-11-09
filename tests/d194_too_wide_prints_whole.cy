// trace: D194
@config(default);
import std;

fn.main [] -> void:
    std::io::fmt_print("[{align: right, width: 3 cols}]{newln}", "longer");
    return void;
end fn.main
