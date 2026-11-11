// `as` is part of the language, not maths.
// trace: D237

struct money:
    cents: i64;
end struct money

op as [$a: money, $b: money] -> money:
    return $a;
end op as

fn.main [none] -> void:
    return void;
end fn.main
