// An operator is never defined for built-in types like `i32`.
// trace: D236
@config(default);

struct money:
    cents: i64;
end struct money

op + [$a: i32, $b: i32] -> i32:
    return 0;
end op +

fn.main [] -> void:
    return void;
end fn.main
