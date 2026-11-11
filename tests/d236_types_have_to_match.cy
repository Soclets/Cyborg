// An operator applies only where its values' types match.
// trace: D236

struct money:
    cents: i64;
end struct money

op + [$a: money, $b: money] -> money:
    return money[cents = $a.cents + $b.cents];
end op +

fn.main [none] -> void:
    let $a: money = money[cents = 1];
    let $n: i64 = 2;
    let $c: money = $a + $n;
    return void;
end fn.main
