// 9.4's example. The oracle builds for no chip, so it follows `else:`, as a build for a chip no branch names would.
// trace: D233
import std;
import os;

fn.main [none] returns void:
    let mut $out: os::stdout.unbuffered = os::stdout.unbuffered();
    let $a: i64 = 6;
    let $b: i64 = 7;
    let $c: i64 = 8;
    on.target aarch64:
        shield mul_add:
            reason "one madd instruction computes a times b plus c"
            inline.asm:
                reads: [x0 = $a, x1 = $b, x2 = $c];
                writes: [x0 => $r];
                clobbers: [none];
                madd x0, x0, x1, x2
            end inline.asm
        end shield mul_add
    else:
        let $r: i64 = $a x $b + $c;
    end on.target
    std::io::fmt_print(Lend.mut $out, "{}{newln}", $r);
    return void;
end fn.main
