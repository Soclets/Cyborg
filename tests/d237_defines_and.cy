// `and` stops early, which an operator couldn't promise, so it can't be defined.
// trace: D237, D124

struct money:
    cents: i64;
end struct money

op and [$a: money, $b: money] returns bool:
    return true;
end op and

fn.main [none] returns void:
    return void;
end fn.main
