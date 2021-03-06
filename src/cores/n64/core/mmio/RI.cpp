#include <n64/core/mmio/RI.hpp>
#include <util.hpp>

namespace n64 {
auto RI::Read(u32 addr) const -> u32 {
  switch(addr) {
    case 0x04700000: return mode;
    case 0x04700004: return config;
    case 0x0470000C: return select;
    case 0x04700010: return refresh;
    default: util::panic("Unhandled RI[{:08X}] read\n", addr); return 0;
  }
}

void RI::Write(u32 addr, u32 val) {
  switch(addr) {
    case 0x04700000: mode = val; break;
    case 0x04700004: config = val; break;
    case 0x0470000C: select = val; break;
    case 0x04700010: refresh = val; break;
    default: util::panic("Unhandled RI[{:08X}] read\n", addr);
  }
}

}
