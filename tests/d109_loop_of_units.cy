// trace: D109
@config(default);

fam.unit money:
    money.cents = 100 money.dollars;
    money.dollars = 100 money.cents;
end fam.unit money

fn.main [] -> void:
    return void;
end fn.main
