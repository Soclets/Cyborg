// Contracts, a pure function and baked constants working together.
// trace: D29, D53, D84, D86, D91
import std;
import os;

const $week: i32 = days(7);

fn.pure.baked days [$n: i32] returns i32:
    return $n x 86_400;
end fn.pure.baked days

fn.pure is_positive [$n: i32] returns bool:
    return $n above 0;
end fn.pure is_positive

fn per_day [$total: i32, $n: i32] returns i32 fails [nothing_to_share]:
    labels [$n as days];
    defaults [$n = 7];
    requires is_positive($n);
    ensures result x $n at_most $total;

    if $total == 0:
        fail nothing_to_share;
    end if
    return $total div.trunc $n;
end fn per_day

fn.main [none] returns void:
    let mut $out: os::stdout.unbuffered = os::stdout.unbuffered();
    std::io::fmt_print(Lend.mut $out, "{}{newln}", $week);
    std::io::fmt_print(Lend.mut $out, "{}{newln}", per_day($week));
    std::io::fmt_print(Lend.mut $out, "{}{newln}", per_day($week, days: 2));
    std::io::fmt_print(Lend.mut $out, "{}{newln}", (per_day(0) ?? -1));
    return void;
end fn.main
