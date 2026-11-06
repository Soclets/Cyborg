// trace: D134, D135
import std;

fam.type.alias sizes:
    sizes.small = i32;
    sizes.large = i64;
end fam.type.alias sizes

fn.main [] -> void:
    let $a: sizes.small = 1;
    let $b: i32 = $a;
    let $c: sizes.large = 2;
    std::println($b as str);
    let $d: i64 = $c + 1;
    std::println("large");
    return void;
end fn.main
