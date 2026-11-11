// A `use my::` line is a path like any other, so an unquoted name matches its last word.
// trace: D248, D150

fam.unit length:
    length.meters: base;
end fam.unit length

use my::length.meters as metres;

fn.main [none] returns void:
    return void;
end fn.main
