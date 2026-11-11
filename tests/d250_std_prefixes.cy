// `std` ships `metric` and `binary`, but which prefixes they list isn't decided.
// trace: D250

fam.unit storage:
    storage.bytes: base with binary;
end fam.unit storage

fn.main [none] returns void:
    return void;
end fn.main
