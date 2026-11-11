// trace: D102, D3
import std;

fn.main [none] returns void:
    let $a: str = "a";
    let $xs: std::list<str>[len.max: 8 items] = [$a];
    return void;
end fn.main
