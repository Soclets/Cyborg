// Conversions are spelled with `as` now, and dollars to cents is an exact `x 100`.
// trace: D109, D108, D255

fam.unit money:
    money.cents: base;
    money.dollars set_to 100 money.cents;
end fam.unit money

fn.main [none] returns void:
    let $d: i32[unit: money.dollars] set_to 2 money.dollars;
    let $c: i32[unit: money.cents] set_to $d as i32[unit: money.cents];
    return void;
end fn.main
