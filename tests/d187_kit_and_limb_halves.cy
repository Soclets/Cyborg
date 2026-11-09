// trace: D187, D155
@config(default);
import std;
import @std;

use @std::inline.asm as asm;

fn.main [] -> void:
    std::io::fmt_print("both halves{newln}");
    return void;
end fn.main
