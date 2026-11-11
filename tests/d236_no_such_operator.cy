// A struct has only the operators it defines.
// trace: D236

struct money:
    cents: i64;
end struct money

fn.main [none] returns void:
    let $a: money set_to money[cents set_to 1];
    let $b: money set_to money[cents set_to 2];
    let $c: money set_to $a + $b;
    return void;
end fn.main
