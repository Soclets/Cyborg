// A type with no unit has bounds with none, the same as any value given to it.
// trace: D260

fam.unit time:
    time.seconds: base;
end fam.unit time

fn.main [none] returns void:
    let $n: i32[min: 0 time.seconds, max: 9] = 5;
    return void;
end fn.main
