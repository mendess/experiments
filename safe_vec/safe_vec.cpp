#include <array>
#include <iostream>
#include <new>
#include <vector>

auto constexpr SIZE = 16384;

auto loop() -> std::pair<size_t, std::bad_alloc> {
    auto v = std::vector<int>();
    try {
        for (;;) {
            v.push_back(0);
        }
    } catch (std::bad_alloc e) {
        return {v.size(), e};
    }
    throw "unreachable";
}

auto main(int argc, char** argv) -> int {
    auto [len, e] = loop();
    std::cout << " [cpp] we got to " << len
              << " before the os kicked us out because of " << e.what()
              << std::endl;
}
