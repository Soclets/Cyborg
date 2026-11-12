// `<` compares, so it gives a `bool`.
// trace: D237

struct money:
    cents: i64;
end struct money

op below [$a: Lent money, $b: Lent money] -> i64:
    return 0;
end op below

fn.main [none] -> void:
    return void;
end fn.main
