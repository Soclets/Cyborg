// trace: D2
import std;

struct box:
    s: str;
end struct box

fn keep [$s: Given str] -> str:
    return Give $s;
end fn keep

fn.main [] -> void:
    let $b: box = box[s = "a"];
    let $k: str = keep(Copy $b.s);
    std::println(Lend $k, Lend $b.s);
    return void;
end fn.main
