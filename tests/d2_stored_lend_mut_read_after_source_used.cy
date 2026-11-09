// `Lend.mut` means nothing else uses it meanwhile.
// trace: D2
@config(default);

struct reading:
    celsius: i32;
end struct reading

fn.main [] -> void:
    let mut $r: reading = reading[celsius = 21];
    let $b: Lend.mut from $r reading = Lend.mut $r;
    let $n: i32 = $r.celsius;
    let $m: i32 = $b.celsius;
    return void;
end fn.main
