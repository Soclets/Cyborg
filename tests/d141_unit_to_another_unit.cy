// Conversions are spelled with `as` now, and dollars to cents is an exact `x 100`.
// trace: D141, D109, D255
@config(default);

fam.unit money:
    money.cents: base;
    money.dollars = 100 money.cents;
end fam.unit money

fn.main [] -> void:
    let $d: i32.money.dollars = 2 money.dollars;
    let $c: i32.money.cents = $d as i32.money.cents;
    return void;
end fn.main
