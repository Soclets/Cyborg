// trace: D47

fam.failure file_failed:
    file_failed.denied;
end fam.failure file_failed

fam.failure net_failed:
    net_failed.denied;
end fam.failure net_failed

fn load [none] -> i32 fails [file_failed, net_failed]:
    return 0;
end fn load

fn.main [none] -> void:
    return void;
end fn.main
