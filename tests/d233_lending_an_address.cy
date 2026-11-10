// Lending an address into a register isn't designed.
// trace: D233
@config(default);
import std;

fn.main [] -> void:
    let $a: i64 = 6;
    let $b: i64 = 7;
    on.target aarch64:
        shield load:
            reason "one load"
            inline.asm:
                reads: [x1 = Lend $a];
                writes: [x2 => $r];
                clobbers: [];
                ldr x2, [x1]
            end inline.asm
        end shield load
    end on.target
    return void;
end fn.main
