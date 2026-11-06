// `=` can point at a unit in any family.
// trace: D109
import std;

fam.unit money:
    money.cents: base;
    money.euro_cents: base;
end fam.unit money

fam.unit cash:
    cash.dollars = 100 money.cents;
end fam.unit cash

fn.main [] -> void:
    let $a: i32.cash.dollars = 2 cash.dollars;
    std::println($a as str);
    return void;
end fn.main
