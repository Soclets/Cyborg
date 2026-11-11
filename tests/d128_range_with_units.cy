// trace: D128, D100

fam.unit time:
    time.seconds: base;
end fam.unit time

fn.main [none] returns void:
    for $i in range[from: 1 time.seconds, to: 5 time.seconds]:
        return void;
    end for
    return void;
end fn.main
