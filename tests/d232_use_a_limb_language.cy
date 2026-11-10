// `use` doesn't work on the base language, so every `inline.…` stays in full, a limb's included.
// trace: D232, D227
@config(default);

use inline.@llvm::ir as ir;

fn.main [] -> void:
    shield low:
        reason "talks to the chip"
        ir();
    end shield low
    return void;
end fn.main
