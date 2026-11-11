// A type's ordering is `op below`, so `op <` is refused with that fix.
// trace: D278, D237
struct money:
    cents: i64;
end struct money

op < [$a: money, $b: money] returns bool:
    return $a.cents below $b.cents;
end op <

fn.main [none] returns void:
    return void;
end fn.main
