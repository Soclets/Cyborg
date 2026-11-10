// A type's own `mod` isn't designed.
// trace: D237
@config(default);

struct money:
    cents: i64;
end struct money

op mod.floor [$a: money, $b: i64] -> money:
    return money[cents = $a.cents mod.floor $b];
end op mod.floor

fn.main [] -> void:
    return void;
end fn.main
