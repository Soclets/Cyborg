// `small` is its own struct here, so the family's `sizes.small` doesn't come into it.
// trace: D134, D57
import std;

struct small:
    n: i32;
end struct small

fam.type.alias sizes:
    sizes.small = i32;
end fam.type.alias sizes

fn.main [] -> void:
    let $a: small = small[n = 1];
    std::println($a.n as str);
    return void;
end fn.main
