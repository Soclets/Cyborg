// trace: D130, D86
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
    std::println(take(Lend $ok) as str);
    let $bad: box = box[n = 0];
    std::println(take(Lend $bad) as str);
    return void;
end fn.main
