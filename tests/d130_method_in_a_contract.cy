// trace: D130, D86
@config(default);
import std;

struct box:
    n: i32;
end struct box

fn.pure box.%positive [$b: box] -> bool:
    return $b.n > 0;
end fn.pure box.%positive

fn take [$b: box] -> i32:
    requires $b.%positive();

    return $b.n;
end fn take

fn.main [] -> void:
    let $ok: box = box[n = 1];
    std::io::fmt_print("{}{newln}", take(Lend $ok));
    let $bad: box = box[n = 0];
    std::io::fmt_print("{}{newln}", take(Lend $bad));
    return void;
end fn.main
