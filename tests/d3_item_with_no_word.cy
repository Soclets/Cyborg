// trace: D102, D3
@config(default);
import std;

fn.main [] -> void:
    let $a: str = "a";
    let $xs: std::list<str>[len.max: 8 items] = [$a];
    return void;
end fn.main
