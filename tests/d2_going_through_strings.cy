// trace: D2
import std;

fn.main [] -> void:
    let $xs: std::array<str>[len: 2 items] = ["a", "b"];
    for $s in $xs:
        std::println(Lend $s);
    end for
    return void;
end fn.main
