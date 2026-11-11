// `>` follows from `<`, so a type doesn't write it.
// trace: D237

struct money:
    cents: i64;
end struct money

op above [$a: money, $b: money] -> bool:
    return $a.cents above $b.cents;
end op >

fn.main [none] -> void:
    return void;
end fn.main
