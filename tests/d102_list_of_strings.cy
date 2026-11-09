// Items that aren't plain numbers show their hand-off too.
// trace: D102, D3
import std;
@config(default);

fn.main [] -> void:
    let $a: str = "a";
    let $b: str = "b";
    let $xs: std::list<str>[len.max: 8 items] = [Give $a, Copy $b, "c"];
    for $s in $xs:
        std::println(Lend $s);
    end for
    std::println(Lend $b);
    return void;
end fn.main
