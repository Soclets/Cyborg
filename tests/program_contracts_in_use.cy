// Contracts, a pure function and baked constants working together.
// trace: D29, D53, D84, D86, D91
import std;

const $week: i32 = days(7);

fn.pure.baked days [$n: i32] -> i32:
    return $n x 86_400;
end fn.pure.baked days

fn.pure is_positive [$n: i32] -> bool:
    return $n > 0;
end fn.pure is_positive

fn per_day [$total: i32, $n: i32] -> i32:
    labels [$n as days];
    defaults [$n = 7];
    requires is_positive($n);
    ensures result x $n at_most $total;
    fails [nothing_to_share];

    if $total == 0:
        fail nothing_to_share;
    end if
    return $total div.trunc $n;
end fn per_day

fn.main [] -> void:
    std::println($week as str);
    std::println(per_day($week) as str);
    std::println(per_day($week, days: 2) as str);
    std::println((per_day(0) ?? -1) as str);
    return void;
end fn.main
