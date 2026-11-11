// A type's own bit operators aren't designed.
// trace: D237

struct money:
    cents: i64;
end struct money

op bit_and [$a: money, $b: money] -> money:
    return $a;
end op bit_and

fn.main [none] -> void:
    return void;
end fn.main
