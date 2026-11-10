// trace: D194
import std;

fn.main [none] -> void:
    std::io::fmt_print("[{align: left, width: 6 cols}]{newln}", "ab");
    std::io::fmt_print("[{align: right, width: 6 cols}]{newln}", "ab");
    std::io::fmt_print("[{align: center, width: 6 cols}]{newln}", "ab");
    return void;
end fn.main
