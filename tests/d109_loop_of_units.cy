// trace: D109

fam.unit money:
    money.cents set_to 100 money.dollars;
    money.dollars set_to 100 money.cents;
end fam.unit money

fn.main [none] returns void:
    return void;
end fn.main
