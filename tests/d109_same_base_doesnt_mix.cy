// How a conversion is written isn't decided, so two units with one base still don't add.
// trace: D109, D100
narrowings.unit money:
    money.cents: base;
    money.dollars = 100 money.cents;
end narrowings.unit money

fn.main [] -> void:
    let $a: i32.money.dollars = 2 money.dollars;
    let $b: i32.money.dollars = $a + 1 money.cents;
    return void;
end fn.main
