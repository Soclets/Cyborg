// trace: D164
@config(default);

fn.main [] -> void:
    let $a: str = "a";
    let $b: str = $a div $a;
    return void;
end fn.main
