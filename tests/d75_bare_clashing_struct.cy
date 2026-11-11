// trace: D75

struct my::div:
    n: i32;
end struct my::div

fn.main [none] returns void:
    let $thing: my::div set_to div[n set_to 9];
    return void;
end fn.main
