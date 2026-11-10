// `>` follows from `<`, so a type doesn't write it.
// trace: D237
@config(default);

struct money:
    cents: i64;
end struct money

op > [$a: money, $b: money] -> bool:
    return $a.cents > $b.cents;
end op >

fn.main [] -> void:
    return void;
end fn.main
