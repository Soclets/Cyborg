// Inline assembly is in the base language now, so `@std::inline.asm` is refused with its new spelling.
// trace: D227, D155
@config(default);
import @std;

use @std::inline.asm as asm;

fn.main [] -> void:
    return void;
end fn.main
