// trace: D50, D103
import std;
@config(default);

fn.main [] -> void:
    let $a: str = "hi";
    let $b: str = empty;
    std::io::fmt_print("{}{}!{newln}", $a, $b);
    return void;
end fn.main
