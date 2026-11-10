// A short name for a limb's inline language matches its last word.
// trace: D227
@config(default);

use inline.@llvm::ir as ir;

fn.main [] -> void:
    shield low:
        reason "talks to the chip"
        ir();
    end shield low
    return void;
end fn.main
