// `use` doesn't work on the base language, so the fix removes the line.
// trace: D232, D227

use inline.asm as asm;

fn.main [none] -> void:
    shield low:
        reason "talks to the chip"
        asm();
    end shield low
    return void;
end fn.main
