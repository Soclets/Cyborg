// Inline assembly is base, written `inline.asm`, so the old limb path is refused even in a shield.
// trace: D227, D28, D38

fn.main [none] returns void:
    shield asm:
        reason "needs one instruction"
        @std::inline.asm();
    end shield asm
    return void;
end fn.main
