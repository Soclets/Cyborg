// After a name, `< >` holds sources in backticks, or type parameters touching the name, and
// anything else is refused, since generics' spelling is decided.
// trace: D234, D171, D286
fn first <i32> [$x: i32] -> i32:
    return $x;
end fn first

fn.main [none] -> void:
    return void;
end fn.main
