// An assembly block sits in an `on.target` choice naming its chip.
// trace: D233
import std;

fn.main [none] returns void:
    let $a: i64 set_to 6;
    let $b: i64 set_to 7;
    shield add:
        reason "one add"
        inline.asm:
            reads: [x0 set_to $a, x1 set_to $b];
            writes: [x2 => $r];
            clobbers: [none];
            add x2, x0, x1
        end inline.asm
    end shield add
    return void;
end fn.main
