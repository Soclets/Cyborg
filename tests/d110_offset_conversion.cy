// How an offset is written isn't decided.
// trace: D110

fam.unit heat:
    heat.kelvin: base;
    heat.celsius = 1 heat.kelvin + 273;
end fam.unit heat

fn.main [none] -> void:
    return void;
end fn.main
