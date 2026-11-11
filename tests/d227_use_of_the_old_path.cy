// Inline assembly is in the base language now, so `@std::inline.asm` is refused with its new spelling.
// trace: D227, D155
import @std;

use @std::inline.asm as asm;

fn.main [none] returns void:
    return void;
end fn.main
