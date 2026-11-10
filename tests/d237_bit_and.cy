// A type's own bit operators aren't designed.
// trace: D237
@config(default);

struct money:
    cents: i64;
end struct money

op bit_and [$a: money, $b: money] -> money:
    return $a;
end op bit_and

fn.main [] -> void:
    return void;
end fn.main
