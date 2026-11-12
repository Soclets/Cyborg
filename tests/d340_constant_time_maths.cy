// What takes the same time for every value works on a secret: the bit operators, the word
// forms of `+`, shifts by a public amount, comparisons, and `.%pick`. Mixed with a plain value,
// a secret spreads, and it becomes public only with `.%reveal()`, in a shield.
// trace: D339, D340, D342, D361
import std;
import os;

fn mix [$key: u32.secret, $nonce: u32] -> u32.secret:
    let $x: u32.secret = $key xor.bit $nonce;
    let $y: u32.secret = $x plus.wrap 7;
    let $z: u32.secret = ($y shl 2) or.bit 1;
    let $same: bool.secret = $x == $nonce;
    return $same.%pick(if_true: $z, if_false: not.bit $z);
end fn mix

fn.main [none] -> void:
    let mut $out: os::stdout.unbuffered = os::stdout.unbuffered();
    let $result: u32.secret = mix(5, 9);
    let $flag: bool.secret = ($result == 0) or.both not ($result above 7);
    shield publish:
        reason "the result of this test is safe to publish"
        let $plain: u32 = $result.%reveal();
        let $flag_plain: bool = $flag.%reveal();
        std::io::fmt_print(Lend.mut $out, "{$plain}{newln}");
        if $flag_plain:
            std::io::fmt_print(Lend.mut $out, "flag{newln}");
        end if
    end shield publish
    return void;
end fn.main
