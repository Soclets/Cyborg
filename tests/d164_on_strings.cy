// trace: D164

fn.main [none] returns void:
    let $a: str set_to "a";
    let $b: str set_to $a div $a;
    return void;
end fn.main
