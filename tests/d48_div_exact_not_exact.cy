// What `div.exact` does when it isn't isn't decided.
// trace: D48
@config(default);

fn.main [] -> void:
    let $a: i32 = 7 div.exact 2;
    return void;
end fn.main
