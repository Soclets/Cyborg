// An assembly block sits in an `on.target` choice naming its chip.
// trace: D233
import std;

fn.main [none] -> void:
    let $a: i64 = 6;
    let $b: i64 = 7;
    shield add:
        reason "one add"
        inline.asm:
            reads: [x0 = $a, x1 = $b];
            writes: [x2 => $r];
            clobbers: [none];
            add x2, x0, x1
        end inline.asm
    end shield add
    return void;
end fn.main
