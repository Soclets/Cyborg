// trace: D187, D155
import std;
import @std;

use @std::inline.asm as asm;
@config(default);

fn.main [] -> void:
    std::io::fmt_print("both halves{newln}");
    return void;
end fn.main
