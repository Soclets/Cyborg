// Two families have a `small`, so there's no one fix to show.
// trace: D134, D16

fam.type.alias sizes:
    sizes.small = i32;
end fam.type.alias sizes

fam.type.alias shoes:
    shoes.small = i32;
end fam.type.alias shoes

fn.main [none] returns void:
    let $a: small = 1;
    return void;
end fn.main
