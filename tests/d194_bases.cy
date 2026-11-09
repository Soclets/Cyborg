// trace: D194
import std;
@config(default);

fn.main [] -> void:
    std::io::fmt_print("{base: bin} {base: oct} {base: hex} {base: hex.upper}{newln}", 10, 10, 255, 255);
    return void;
end fn.main
