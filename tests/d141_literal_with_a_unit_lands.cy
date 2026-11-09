// trace: D141, D101
import std;
@config(default);

fn.main [] -> void:
    let $t: u64.items = 5 items;
    std::io::fmt_print("{}{newln}", $t);
    return void;
end fn.main
