// An item that isn't a plain number shows its hand-off like anything else.
// trace: D133, D3
import std;

fn.main [] -> void:
    let $a: std::array<str>[len: 1 items] = ["hi"];
    let $s: str = $a[pos: 1];
    return void;
end fn.main
