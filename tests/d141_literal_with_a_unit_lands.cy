// trace: D141, D101
import std;
@config(default);

fn.main [] -> void:
    let $t: u64.items = 5 items;
    std::println($t as str);
    return void;
end fn.main
