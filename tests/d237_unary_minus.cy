// A type's own unary `-` isn't designed.
// trace: D237

struct money:
    cents: i64;
end struct money

op - [$a: money] returns money:
    return money[cents set_to -$a.cents];
end op -

fn.main [none] returns void:
    return void;
end fn.main
