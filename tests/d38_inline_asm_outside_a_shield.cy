// Inline assembly is only allowed in a shield.
// trace: D38, D227
@config(default);

fn.main [] -> void:
    inline.asm();
    return void;
end fn.main
