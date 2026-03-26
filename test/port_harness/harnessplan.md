# C++ Reference Harness Plan

## Purpose
Build the original C++ Ratpack code as a standalone test harness that produces
reference outputs for every function. These outputs serve as the ground truth
for validating the Rust port.

## Architecture
```
test/port_harness/
├── harnessplan.md         ← this file
├── CMakeLists.txt         ← CMake build for the C++ harness
├── ratpack_harness.cpp    ← CLI wrapper: reads JSON test cases, calls C++ functions
└── build/                 ← build output (gitignored)
```

## How It Works

1. **ratpack_harness.cpp** is a standalone C++ program that:
   - Includes Ratpack headers from `code2port/src/CalcManager/Ratpack/`
   - Reads test case JSON from stdin (one per line)
   - For each test case, calls the specified C++ function with the given arguments
   - Outputs the result as JSON to stdout

2. **Input format** (one JSON object per line):
   ```json
   {"function": "add_rat", "args": ["3/1", "4/1"], "radix": 10, "precision": 128}
   ```

3. **Output format**:
   ```json
   {"function": "add_rat", "result": "7/1", "error": null}
   ```
   Or on error:
   ```json
   {"function": "log_rat", "result": null, "error": "CALC_E_DOMAIN"}
   ```

## Build Requirements
- Visual Studio 2022 (Enterprise available at `C:\Program Files\Microsoft Visual Studio\2022\Enterprise`)
- CMake (use VS bundled cmake or standalone)
- No external dependencies — Ratpack is pure C++ with STL only

## Supported Functions
All public Ratpack functions listed in ratpak.h:
- Arithmetic: addrat, subrat, mulrat, divrat, remrat, modrat, ratpowi32
- Support: intrat, fracrat, gcdrat, trimit, scale, scale2pi
- Conversion: StringToNumber, NumberToString, RatToString, StringToRat
- Base: nRadixxtonum, numtonRadixx
- Exp/Log: exprat, lograt, log10rat, powrat, rootrat
- Trig: sinrat, cosrat, tanrat, sinanglerat, cosanglerat, tananglerat
- Inverse trig: asinrat, acosrat, atanrat, asinanglerat, acosanglerat, atananglerat
- Hyperbolic: sinhrat, coshrat, tanhrat
- Inverse hyperbolic: asinhrat, acoshrat, atanhrat
- Factorial: factrat
- Logic: andrat, orrat, xorrat, lshrat, rshrat

## Build Instructions
```powershell
cd test/port_harness
cmake -B build -G "Visual Studio 17 2022" -A x64
cmake --build build --config Release
```
