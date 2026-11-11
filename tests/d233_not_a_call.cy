// Assembly is a block, `inline.asm:`, not a call.
// trace: D233, D227

fn.main [none] returns void:
    shield low:
        reason "talks to the chip"
        inline.asm();
    end shield low
    return void;
end fn.main
