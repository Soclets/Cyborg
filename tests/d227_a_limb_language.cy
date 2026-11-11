// A limb adds its own inline language as `inline.@limb::lang`.
// trace: D227

fn.main [none] returns void:
    shield low:
        reason "talks to the chip"
        inline.@llvm::ir();
    end shield low
    return void;
end fn.main
