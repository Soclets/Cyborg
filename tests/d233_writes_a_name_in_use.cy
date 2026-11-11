// `writes:` declares its variables as a `let` would, so the name has to be new.
// trace: D233
import std;

fn.main [none] returns void:
    let $a: i64 = 6;
    let $b: i64 = 7;
    let $c: i64 = 8;
    on.target aarch64:
        shield add:
            reason "one add"
            inline.asm:
                reads: [x0 = $a, x1 = $b];
                writes: [x2 => $c];
                clobbers: [none];
                add x2, x0, x1
            end inline.asm
        end shield add
    end on.target
    return void;
end fn.main
