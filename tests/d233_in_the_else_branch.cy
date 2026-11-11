// `else:` names no chip, so an assembly block can't go there.
// trace: D233
import std;

fn.main [none] returns void:
    let $a: i64 set_to 6;
    let $b: i64 set_to 7;
    on.target aarch64:
        let $r: i64 set_to $a + $b;
    else:
        shield add:
            reason "one add"
            inline.asm:
                reads: [x0 set_to $a, x1 set_to $b];
                writes: [x2 => $s];
                clobbers: [none];
                add x2, x0, x1
            end inline.asm
        end shield add
    end on.target
    return void;
end fn.main
