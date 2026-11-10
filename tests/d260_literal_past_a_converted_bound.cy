// A literal past a bound written in another unit is refused when compiling.
// trace: D260, D20

fam.unit length:
    length.meters: base;
    length.kilometers = 1000 length.meters;
end fam.unit length

fn.main [none] -> void:
    let $far: i32[unit: length.meters, min: 0 length.meters, max: 1 length.kilometers] = 1001 length.meters;
    return void;
end fn.main
