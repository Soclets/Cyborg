// `at_least` follows from `<` too.
// trace: D237
@config(default);

struct money:
    cents: i64;
end struct money

op at_least [$a: money, $b: money] -> bool:
    return $a.cents at_least $b.cents;
end op at_least

fn.main [] -> void:
    return void;
end fn.main
