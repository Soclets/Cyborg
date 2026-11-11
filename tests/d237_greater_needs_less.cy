// `>` follows from `<`, so it needs the type's `op <`.
// trace: D237

struct money:
    cents: i64;
end struct money

op == [$a: money, $b: money] returns bool:
    return $a.cents == $b.cents;
end op ==

fn.main [none] returns void:
    let $a: money set_to money[cents set_to 1];
    let $b: money set_to money[cents set_to 2];
    let $c: bool set_to $a above $b;
    return void;
end fn.main
