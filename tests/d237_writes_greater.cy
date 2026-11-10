// `>` follows from `<`, so a type doesn't write it.
// trace: D237

struct money:
    cents: i64;
end struct money

op > [$a: money, $b: money] -> bool:
    return $a.cents > $b.cents;
end op >

fn.main [none] -> void:
    return void;
end fn.main
