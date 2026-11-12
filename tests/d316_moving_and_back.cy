// A pointer moved one item on and one item back points at its variable again.
// trace: D316
import std;
import os;

fn.main [none] -> void:
    let mut $out: os::stdout.unbuffered = os::stdout.unbuffered();
    let mut $count: u32 = 9;
    shield move:
        reason "testing a move"
        let $p: ptr.raw.mut<u32> = ptr.raw.mut<u32>.of(Lend.mut $count);
        let $there: ptr.raw.mut<u32> = $p.%moved(offset.items: 1);
        let $back: ptr.raw.mut<u32> = $there.%moved(offset.bytes: -4);
        std::io::fmt_print(Lend.mut $out, "{}{newln}", $back.%read());
    end shield move
    return void;
end fn.main
