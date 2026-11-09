// `std`'s limb part is only reached with `@`, and inline assembly must be in a shield.
// trace: D22, D28, D38
@config(default);

fn.main [] -> void:
    shield asm:
        reason "needs one instruction"
        @std::inline.asm();
    end shield asm
    return void;
end fn.main
