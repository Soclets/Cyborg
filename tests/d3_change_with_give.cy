// trace: D3
import std;

fn.main [] -> void:
    let $s: str = "a";
    let mut $t: str = "b";
    change $t = Give $s;
    std::println(Lend $t);
    return void;
end fn.main
