// The lines after `clobbers:` are raw assembly, so `#` and `[ ]` mean nothing to Cyborg there.
// trace: D233
import std;

fn.main [none] -> void:
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
    std::io::fmt_print("{}{newln}", $r);
    return void;
end fn.main
