// trace: D116, D48, D97
import std;
@config(default);

fn.main [] -> void:
    std::println((7 div.trunc 2) as str);
    std::println((-7 div.trunc 2) as str);
    std::println((-7 div.floor 2) as str);
    std::println((7 div.floor -2) as str);
    std::println((8 div.exact 2) as str);
    std::println((-7 mod.trunc 2) as str);
    std::println((-7 mod.floor 2) as str);
    std::println((7 mod.floor -2) as str);
    return void;
end fn.main
