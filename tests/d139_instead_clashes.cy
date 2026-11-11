// trace: D139, D75

fn my::instead [none] returns i32:
    return 1;
end fn my::instead

fn.main [none] returns void:
    let $a: i32 set_to instead();
    return void;
end fn.main
