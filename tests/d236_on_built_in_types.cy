// An operator is never defined for built-in types like `i32`.
// trace: D236

struct money:
    cents: i64;
end struct money

op + [$a: i32, $b: i32] returns i32:
    return 0;
end op +

fn.main [none] returns void:
    return void;
end fn.main
