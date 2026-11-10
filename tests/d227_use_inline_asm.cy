// `inline.asm` can have a short name too, matching its last word.
// trace: D227
@config(default);

use inline.asm as asm;

fn.main [] -> void:
    shield low:
        reason "talks to the chip"
        asm();
    end shield low
    return void;
end fn.main
