// An integer conversion whose ratio isn't whole, like feet to 9-inch spans, 4 per 3, isn't decided.
// trace: D255

fam.unit length:
    length.inches: base;
    length.feet = 12 length.inches;
    length.spans = 9 length.inches;
end fam.unit length

fn.main [none] returns void:
    let $f: i32[unit: length.feet] = 3 length.feet;
    let $s: i32[unit: length.spans] = $f as.trunc i32[unit: length.spans];
    return void;
end fn.main
