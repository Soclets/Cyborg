// A bare `str` is a view, so `as str` is refused, with the size the type needs as its fix.
// trace: D328, D327
fn.main [none] -> void:
    let $n: i64 = 42;
    let $s: str = $n as str;
    return void;
end fn.main
