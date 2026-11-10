// How a conversion is written isn't decided, so two units with one base still don't add.
// trace: D109, D100

fam.unit money:
    money.cents: base;
    money.dollars = 100 money.cents;
end fam.unit money

fn.main [none] -> void:
    let $a: i32[unit: money.dollars] = 2 money.dollars;
    let $b: i32[unit: money.dollars] = $a + 1 money.cents;
    return void;
end fn.main
