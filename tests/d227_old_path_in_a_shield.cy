// Inline assembly is base, written `inline.asm`, so the old limb path is refused even in a shield.
// trace: D227, D28, D38
@config(default);

fn.main [] -> void:
    shield asm:
        reason "needs one instruction"
        @std::inline.asm();
    end shield asm
    return void;
end fn.main
