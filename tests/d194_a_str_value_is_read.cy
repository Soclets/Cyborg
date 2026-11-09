// A print reads its values, so a `str` takes no word.
// trace: D194, D95
@config(default);
import std;

fn.main [] -> void:
    let $s: str = "hi";
    std::io::fmt_print("{}{newln}", $s);
    return void;
end fn.main
