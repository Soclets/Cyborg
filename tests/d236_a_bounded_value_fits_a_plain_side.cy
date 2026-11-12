// A bounded value goes into a plain one with no check, so it fits a type's own operator whose
// side is the plain type.
// trace: D236, D20
import std;
import os;

struct money:
    cents: i64;
end struct money

op div.floor [$a: Lent money, $b: i64] -> money:
    return money[cents = $a.cents div.floor $b];
end op div.floor

fn.main [none] -> void:
    let mut $out: os::stdout.unbuffered = os::stdout.unbuffered();
    let $parts: i64[min: 1, max: 10] = 4;
    let $price: money = money[cents = 250];
    let $share: money = $price div.floor $parts;
    std::io::fmt_print(Lend.mut $out, "{}{newln}", $share.cents);
    return void;
end fn.main
