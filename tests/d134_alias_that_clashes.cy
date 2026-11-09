// A name of yours that clashes stays bare where it's defined, and says `my::` where it's used.
// trace: D134, D75
import std;
@config(default);

type.alias str = i32;

fn.main [] -> void:
    let $a: my::str = 5;
    let $b: str = "still a string";
    std::println($a as str);
    std::println(Lend $b);
    return void;
end fn.main
