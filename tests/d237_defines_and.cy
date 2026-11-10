// `and` stops early, which an operator couldn't promise, so it can't be defined.
// trace: D237, D124
@config(default);

struct money:
    cents: i64;
end struct money

op and [$a: money, $b: money] -> bool:
    return true;
end op and

fn.main [] -> void:
    return void;
end fn.main
