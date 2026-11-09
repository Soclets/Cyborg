// trace: D188, D155
import pkg std;

use @std::inline.asm as asm;
@config(default);

fn.main [] -> void:
    std::println("both halves");
    return void;
end fn.main
