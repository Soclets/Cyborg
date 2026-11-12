// `change` doesn't stand in for an ownership word.
// trace: D3
import std;

fn.main [none] -> void:
    let $s: str[len.max: 32 bytes] = "a";
    let mut $t: str[len.max: 32 bytes] = "b";
    change $t = $s;
    return void;
end fn.main
