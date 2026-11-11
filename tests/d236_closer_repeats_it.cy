// The closer repeats the operator.
// trace: D236, D33

struct money:
    cents: i64;
end struct money

op + [$a: money, $b: money] returns money:
    return money[cents set_to $a.cents + $b.cents];
end op x

fn.main [none] returns void:
    return void;
end fn.main
