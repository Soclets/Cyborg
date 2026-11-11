// An integer type's bound converts exactly, so 1500 metres on kilometres is refused, not rounded.
// trace: D260

fam.unit length:
    length.meters: base;
    length.kilometers set_to 1000 length.meters;
end fam.unit length

fn.main [none] returns void:
    let $k: i32[unit: length.kilometers, min: 0 length.kilometers, max: 1500 length.meters] set_to 1 length.kilometers;
    return void;
end fn.main
