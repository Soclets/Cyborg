// An assembly block must be inside a shield.
// trace: D233, D38
import std;

fn.main [none] returns void:
    let $a: i64 set_to 6;
    let $b: i64 set_to 7;
    on.target aarch64:
        inline.asm:
            reads: [x0 set_to $a, x1 set_to $b];
            writes: [x2 => $r];
            clobbers: [none];
            add x2, x0, x1
        end inline.asm
    end on.target
    return void;
end fn.main
