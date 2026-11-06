// trace: D75
struct div:
    n: i32;
end struct div

fn.main [] -> void:
    let $thing: my::div = div[n = 9];
    return void;
end fn.main
