// trace: D2
import std;
@config(default);

struct box:
    s: str;
end struct box

fn keep [$s: Given str] -> str:
    return Give $s;
end fn keep

fn.main [] -> void:
    let $b: box = box[s = "a"];
    let $k: str = keep(Copy $b.s);
    std::io::fmt_print("{}{}{newln}", $k, $b.s);
    return void;
end fn.main
