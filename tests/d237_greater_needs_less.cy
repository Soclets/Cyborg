// `>` follows from `<`, so it needs the type's `op <`.
// trace: D237

struct money:
    cents: i64;
end struct money

op == [$a: money, $b: money] -> bool:
    return $a.cents == $b.cents;
end op ==

fn.main [none] -> void:
    let $a: money = money[cents = 1];
    let $b: money = money[cents = 2];
    let $c: bool = $a > $b;
    return void;
end fn.main
