// Inline assembly needs no import, and how its code is written isn't designed.
// trace: D227
@config(default);

fn.main [] -> void:
    shield low:
        reason "talks to the chip"
        inline.asm();
    end shield low
    return void;
end fn.main
