// `change` doesn't stand in for an ownership word.
// trace: D3
import std;

fn.main [none] returns void:
    let $s: str set_to "a";
    let mut $t: str set_to "b";
    change $t set_to $s;
    return void;
end fn.main
