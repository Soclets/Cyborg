// trace: D176, D171
@config(default);

struct reading:
    celsius: i32;
end struct reading

union.tagged lookup <`text`>:
    found[n: i32]
    missing
end union.tagged lookup

fn.main [] -> void:
    return void;
end fn.main
