// Both work on lists and arrays, and a `str` is neither.
// trace: D133, D50

fn.main [none] returns void:
    let $s: str = "hi";
    let $n: i32 = $s[pos: 1];
    return void;
end fn.main
