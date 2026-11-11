// With no `else:`, a build for another chip, like the oracle's, gets nothing from the choice.
// trace: D233
import std;
import os;

fn.main [none] -> void:
    let mut $out: os::stdout.unbuffered = os::stdout.unbuffered();
    let $a: i64 = 6;
    let $b: i64 = 7;
    on.target aarch64:
        shield add:
            reason "one add"
            inline.asm:
                reads: [x0 = $a, x1 = $b];
                writes: [x2 => $r];
                clobbers: [none];
                add x2, x0, x1
            end inline.asm
        end shield add
    end on.target
    std::io::fmt_print(Lend.mut $out, "{}{newln}", $r);
    return void;
end fn.main
