// Only `inline.asm` is in the base language.
// trace: D227
@config(default);

fn.main [] -> void:
    shield low:
        reason "talks to the chip"
        inline.c();
    end shield low
    return void;
end fn.main
