// `<` compares, so it gives a `bool`.
// trace: D237

struct money:
    cents: i64;
end struct money

op < [$a: money, $b: money] -> i64:
    return 0;
end op <

fn.main [none] -> void:
    return void;
end fn.main
