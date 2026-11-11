// An operator applies only where its values' types match.
// trace: D236

struct money:
    cents: i64;
end struct money

op + [$a: money, $b: money] returns money:
    return money[cents set_to $a.cents + $b.cents];
end op +

fn.main [none] returns void:
    let $a: money set_to money[cents set_to 1];
    let $n: i64 set_to 2;
    let $c: money set_to $a + $n;
    return void;
end fn.main
