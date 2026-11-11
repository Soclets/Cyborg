// An item that isn't a plain number shows its hand-off like anything else.
// trace: D133, D3
import std;

fn.main [none] returns void:
    let $a: std::array<str>[len: 1 items] set_to ["hi"];
    let $s: str set_to $a[pos: 1];
    return void;
end fn.main
