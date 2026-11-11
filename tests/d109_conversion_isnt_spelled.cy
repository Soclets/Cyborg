// Conversions are spelled with `as` now, and dollars to cents is an exact `x 100`.
// trace: D109, D108, D255

fam.unit money:
    money.cents: base;
    money.dollars = 100 money.cents;
end fam.unit money

fn.main [none] returns void:
    let $d: i32[unit: money.dollars] = 2 money.dollars;
    let $c: i32[unit: money.cents] = $d as i32[unit: money.cents];
    return void;
end fn.main
