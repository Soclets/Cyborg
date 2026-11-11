// A unit's conversion is decided, but a scale past what the oracle's 128-bit numbers hold
// exactly is something it doesn't handle yet, so it says `NotBuiltYet`, not `NotDesignedYet`.
// trace: D286, D109
fam.unit length:
    length.meters: base;
    length.huge = 1000000000000000000000000000000000000000000 length.meters;
end fam.unit length

fn.main [none] -> void:
    let $a: i32[unit: length.meters] = 1 length.meters;
    return void;
end fn.main
