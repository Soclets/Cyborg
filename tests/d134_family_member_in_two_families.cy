// Two families have a `small`, so there's no one fix to show.
// trace: D134, D16

fam.type.alias sizes:
    sizes.small set_to i32;
end fam.type.alias sizes

fam.type.alias shoes:
    shoes.small set_to i32;
end fam.type.alias shoes

fn.main [none] returns void:
    let $a: small set_to 1;
    return void;
end fn.main
