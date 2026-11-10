// trace: D141, D101
@config(default);
import std;

fn.main [] -> void:
    let $t: u64[unit: items] = 5 items;
    std::io::fmt_print("{}{newln}", $t);
    return void;
end fn.main
