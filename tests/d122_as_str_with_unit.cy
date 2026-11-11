// How a unit prints isn't decided.
// trace: D122

fam.unit time:
    time.seconds: base;
end fam.unit time

fn.main [none] returns void:
    let $a: i32[unit: time.seconds] set_to 1 time.seconds;
    let $s: str set_to $a as str.with_unit;
    return void;
end fn.main
