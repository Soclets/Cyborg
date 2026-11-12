// A type's own bit operators aren't designed.
// trace: D237

struct money:
    cents: i64;
end struct money

op and.bit [$a: money, $b: money] -> money:
    return $a;
end op and.bit

fn.main [none] -> void:
    return void;
end fn.main
