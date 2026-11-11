// The lines after `clobbers:` are raw assembly, so `#` and `[ ]` mean nothing to Cyborg there.
// trace: D233
import std;
import os;

fn.main [none] returns void:
    let mut $out: os::stdout.unbuffered = os::stdout.unbuffered();
    let $a: i64 = 6;
    let $b: i64 = 7;
    let $c: i64 = 8;
    on.target x86_64:
        shield add:
            reason "one add"
            inline.asm:
                reads: [rax = $a, rbx = $b];
                writes: [rcx => $r];
                clobbers: [rdx];
                mov rcx, [rax + 8]   # load
                add rcx, rbx

                xor rdx, rdx
            end inline.asm
        end shield add
    else:
        let $r: i64 = $a + $b;
    end on.target
    std::io::fmt_print(Lend.mut $out, "{}{newln}", $r);
    return void;
end fn.main
