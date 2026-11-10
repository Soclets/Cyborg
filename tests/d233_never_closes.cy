// An assembly block closes with `end inline.asm`.
// trace: D233
@config(default);

fn.main [] -> void:
    on.target aarch64:
        shield add:
            reason "one add"
            inline.asm:
                reads: [];
                writes: [];
                clobbers: [];
                nop
        end shield add
    end on.target
    return void;
end fn.main
