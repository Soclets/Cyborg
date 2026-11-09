// trace: D187, D155
import std;
import @std;

use @std::inline.asm as asm;
@config(default);

fn.main [] -> void:
    std::println("both halves");
    return void;
end fn.main
