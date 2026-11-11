// trace: D128
import std;
import os;

fn.main [none] returns void:
    let mut $out: os::stdout.unbuffered = os::stdout.unbuffered();
    let $start: i64 = 2;
    let $stop: i64 = 4;
    for $i in range[from: $start, to: $stop]:
        let $copy: i64 = $i;
        std::io::fmt_print(Lend.mut $out, "i64{newln}");
    end for
    let $bounded: i32[min: 0, max: 9] = 1;
    for $i in range[from: $bounded, to: 2]:
        std::io::fmt_print(Lend.mut $out, "{}{newln}", ($i + 1));
    end for
    return void;
end fn.main
