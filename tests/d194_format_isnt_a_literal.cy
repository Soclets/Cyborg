// trace: D194
@config(default);
import std;

fn.main [] -> void:
    let $f: str = "hi";
    std::io::fmt_print($f);
    return void;
end fn.main
