// trace: D47
import std;

narrowings.failure file_failed:
    file_failed.denied;
end narrowings.failure file_failed

narrowings.failure net_failed:
    net_failed.denied;
end narrowings.failure net_failed

fn load [] -> i32:
    fails [file_failed.denied, net_failed.denied];

    fail net_failed.denied;
end fn load

fn.main [] -> void:
    let $n: i32 = load() otherwise:
        case file_failed.denied:
            std::println("file");
            return void;
        case net_failed.denied:
            std::println("net");
            return void;
    end otherwise
    return void;
end fn.main
