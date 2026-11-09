// trace: D134
@config(default);

fam.type.alias sizes:
    sizes.small = i32;
end fam.type.alias sizes

fn.main [] -> void:
    let $a: small = 1;
    return void;
end fn.main
