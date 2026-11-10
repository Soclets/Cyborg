// trace: D149
import std;

type.new title = str;

fn.main [none] -> void:
    let $s: str = "a";
    let $t: title = Copy $s as title;
    std::io::fmt_print("{}{newln}", $s);
    return void;
end fn.main
