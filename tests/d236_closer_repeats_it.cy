// The closer repeats the operator.
// trace: D236, D33
@config(default);

struct money:
    cents: i64;
end struct money

op + [$a: money, $b: money] -> money:
    return money[cents = $a.cents + $b.cents];
end op x

fn.main [] -> void:
    return void;
end fn.main
