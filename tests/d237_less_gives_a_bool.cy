// `<` compares, so it gives a `bool`.
// trace: D237

struct money:
    cents: i64;
end struct money

op below [$a: money, $b: money] returns i64:
    return 0;
end op below

fn.main [none] returns void:
    return void;
end fn.main
