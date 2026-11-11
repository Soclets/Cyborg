// A type's own bit operators aren't designed.
// trace: D237

struct money:
    cents: i64;
end struct money

op bit_and [$a: money, $b: money] returns money:
    return $a;
end op bit_and

fn.main [none] returns void:
    return void;
end fn.main
