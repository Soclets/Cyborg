// trace: D194, D197
import std;
@config(default);

fn.main [] -> void:
    std::io::fmt_print("\\ \" \' \{x\} \x41\u00e9\U0001F600{tab}|{newln}");
    return void;
end fn.main
