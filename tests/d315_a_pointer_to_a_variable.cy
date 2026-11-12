// `.of` takes a variable's address, with `Lend.mut` for a pointer that writes, and
// `.%read()` and `.%%write(val:)` go through it, in a shield.
// trace: D314, D315, D226
import std;
import os;

fn.main [none] -> void:
    let mut $out: os::stdout.unbuffered = os::stdout.unbuffered();
    let mut $count: i32 = 5;
    shield poke:
        reason "testing a pointer to a variable"
        let $p: ptr.raw.mut<i32> = ptr.raw.mut<i32>.of(Lend.mut $count);
        $p.%%write(val: 6);
        let $now: i32 = $p.%read();
        std::io::fmt_print(Lend.mut $out, "{$now}{newln}");
    end shield poke
    std::io::fmt_print(Lend.mut $out, "{$count}{newln}");
    return void;
end fn.main
