// trace: D75
@config(default);

struct my::div:
    n: i32;
end struct my::div

fn.main [] -> void:
    let $thing: my::div = div[n = 9];
    return void;
end fn.main
