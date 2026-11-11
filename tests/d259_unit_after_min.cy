// The unit comes first, since a type reads as what it measures, then its range.
// trace: D259, D258

fam.unit length:
    length.meters: base;
end fam.unit length

fn.main [none] returns void:
    let $d: f64.stable[min: 0.0, unit: length.meters] = 5.0 length.meters;
    return void;
end fn.main
