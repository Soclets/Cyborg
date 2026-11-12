// trace: D102, D3
import std;

fn.main [none] -> void:
    let $a: str[len.max: 32 bytes] = "a";
    let $xs: std::coll::list<str[len.max: 32 bytes]>[len.max: 8 items] = [$a];
    return void;
end fn.main
