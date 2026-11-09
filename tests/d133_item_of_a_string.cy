// Both work on lists and arrays, and a `str` is neither.
// trace: D133, D50
@config(default);

fn.main [] -> void:
    let $s: str = "hi";
    let $n: i32 = $s[pos: 1];
    return void;
end fn.main
