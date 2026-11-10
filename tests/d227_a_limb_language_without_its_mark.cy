// A limb's inline language says its limb with `@`.
// trace: D227
@config(default);

fn.main [] -> void:
    shield low:
        reason "talks to the chip"
        inline.llvm::ir();
    end shield low
    return void;
end fn.main
