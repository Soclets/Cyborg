// `change` doesn't stand in for an ownership word.
// trace: D3
import std;
@config(default);

fn.main [] -> void:
    let $s: str = "a";
    let mut $t: str = "b";
    change $t = $s;
    return void;
end fn.main
