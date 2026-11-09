// How a unit prints isn't decided.
// trace: D122
@config(default);

fam.unit time:
    time.seconds: base;
end fam.unit time

fn.main [] -> void:
    let $a: i32.time.seconds = 1 time.seconds;
    let $s: str = $a as str.with_unit;
    return void;
end fn.main
