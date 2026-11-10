// An assembly block must be inside a shield.
// trace: D233, D38
@config(default);
import std;

fn.main [] -> void:
    let $a: i64 = 6;
    let $b: i64 = 7;
    on.target aarch64:
        inline.asm:
            reads: [x0 = $a, x1 = $b];
            writes: [x2 => $r];
            clobbers: [];
            add x2, x0, x1
        end inline.asm
    end on.target
    return void;
end fn.main
