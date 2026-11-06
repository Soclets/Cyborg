// trace: D2, D33
struct reading:
    celsius: i32;
end struct reading

union.tagged maybe:
    none
    some[r: Lend from program reading]
end union.tagged maybe

fn.main [] -> void:
    return void;
end fn.main
