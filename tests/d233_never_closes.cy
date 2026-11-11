// An assembly block closes with `end inline.asm`.
// trace: D233

fn.main [none] -> void:
    on.target aarch64:
        shield add:
            reason "one add"
            inline.asm:
                reads: [none];
                writes: [none];
                clobbers: [none];
                nop
        end shield add
    end on.target
    return void;
end fn.main
