// A print reads its values, so a `str` takes no word.
// trace: D194, D95
import std;

fn.main [none] -> void:
    let $s: str = "hi";
    std::io::fmt_print("{}{newln}", $s);
    return void;
end fn.main
