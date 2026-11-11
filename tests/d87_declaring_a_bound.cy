// A type fits an interface with `satisfies`, not `is`.
// trace: D87, D235

struct reading:
    n: i32;
end struct reading

reading is ordered;

fn.main [none] returns void:
    return void;
end fn.main
