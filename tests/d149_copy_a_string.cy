// trace: D149
import std;
@config(default);

type.new title = str;

fn.main [] -> void:
    let $s: str = "a";
    let $t: title = Copy $s as title;
    std::io::fmt_print("{}{newln}", $s);
    return void;
end fn.main
