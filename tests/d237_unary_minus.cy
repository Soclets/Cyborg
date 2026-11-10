// A type's own unary `-` isn't designed.
// trace: D237

struct money:
    cents: i64;
end struct money

op - [$a: money] -> money:
    return money[cents = -$a.cents];
end op -

fn.main [none] -> void:
    return void;
end fn.main
