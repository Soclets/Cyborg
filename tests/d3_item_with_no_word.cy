// trace: D102, D3
import std;

fn.main [none] returns void:
    let $a: str set_to "a";
    let $xs: std::list<str>[len.max: 8 items] set_to [$a];
    return void;
end fn.main
