// A plain number as a bound on a unit type is refused, with the type's unit as its fix.
// trace: D260, D101

fam.unit time:
    time.seconds: base;
end fam.unit time

fn.main [none] -> void:
    let $t: i32[unit: time.seconds, min: 0, max: 3600 time.seconds] = 60 time.seconds;
    return void;
end fn.main
