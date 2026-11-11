// A struct has only the operators it defines.
// trace: D236

struct money:
    cents: i64;
end struct money

fn.main [none] -> void:
    let $a: money = money[cents = 1];
    let $b: money = money[cents = 2];
    let $c: money = $a + $b;
    return void;
end fn.main
