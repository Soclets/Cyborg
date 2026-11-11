// 9.4's example. The oracle builds for no chip, so it follows `else:`, as a build for a chip no branch names would.
// trace: D233
import std;
import os;

fn.main [none] returns void:
    let mut $out: os::stdout.unbuffered set_to os::stdout.unbuffered();
    let $a: i64 set_to 6;
    let $b: i64 set_to 7;
    let $c: i64 set_to 8;
    on.target aarch64:
        shield mul_add:
            reason "one madd instruction computes a times b plus c"
            inline.asm:
                reads: [x0 set_to $a, x1 set_to $b, x2 set_to $c];
                writes: [x0 => $r];
                clobbers: [none];
                madd x0, x0, x1, x2
            end inline.asm
        end shield mul_add
    else:
        let $r: i64 set_to $a x $b + $c;
    end on.target
    std::io::fmt_print(Lend.mut $out, "{}{newln}", $r);
    return void;
end fn.main
