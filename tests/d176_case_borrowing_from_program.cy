// What can be lent from `program` is still open, in a case as anywhere.
// trace: D176, D2, D33

struct reading:
    celsius: i32;
end struct reading

union.tagged maybe:
    none;
    some[r: Lend from program reading];
end union.tagged maybe

fn.main [none] returns void:
    return void;
end fn.main
