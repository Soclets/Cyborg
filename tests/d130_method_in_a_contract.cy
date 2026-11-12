// trace: D130, D86
import std;
import os;

struct box:
    n: i32;
end struct box

fn.pure box.%positive [$b: Lent box] -> bool:
    return $b.n above 0;
end fn.pure box.%positive

fn take [$b: Lent box] -> i32:
    requires $b.%positive();

    return $b.n;
end fn take

fn.main [none] -> void:
    let mut $out: os::stdout.unbuffered = os::stdout.unbuffered();
    let $ok: box = box[n = 1];
    std::io::fmt_print(Lend.mut $out, "{}{newln}", take(Lend $ok));
    let $bad: box = box[n = 0];
    std::io::fmt_print(Lend.mut $out, "{}{newln}", take(Lend $bad));
    return void;
end fn.main
