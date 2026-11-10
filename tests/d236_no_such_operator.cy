// A struct has only the operators it defines.
// trace: D236
@config(default);

struct money:
    cents: i64;
end struct money

fn.main [] -> void:
    let $a: money = money[cents = 1];
    let $b: money = money[cents = 2];
    let $c: money = $a + $b;
    return void;
end fn.main
