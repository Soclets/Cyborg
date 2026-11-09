// trace: D2
import std;
@config(default);

struct box:
    s: str;
    t: str;
end struct box

fn replace [$s: Lent.mut str] -> void:
    change $s = "changed";
    return void;
end fn replace

fn.main [] -> void:
    let mut $b: box = box[s = "a", t = "b"];
    replace(Lend.mut $b.t);
    std::io::fmt_print("{}{}{newln}", $b.s, $b.t);
    return void;
end fn.main
