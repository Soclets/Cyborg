// An operator shows no word, so a `Given` value isn't designed.
// trace: D236
@config(default);

struct money:
    cents: i64;
end struct money

op + [$a: Given money, $b: money] -> money:
    return money[cents = $a.cents + $b.cents];
end op +

fn.main [] -> void:
    return void;
end fn.main
