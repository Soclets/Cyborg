// trace: D109
narrowings.unit money:
    money.cents = 100 money.dollars;
    money.dollars = 100 money.cents;
end narrowings.unit money

fn.main [] -> void:
    return void;
end fn.main
