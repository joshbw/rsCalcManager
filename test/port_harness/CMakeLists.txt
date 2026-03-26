cmake_minimum_required(VERSION 3.15)
project(ratpack_harness CXX)

set(CMAKE_CXX_STANDARD 17)
set(CMAKE_CXX_STANDARD_REQUIRED ON)

# Ratpack source directory
set(RATPACK_DIR "${CMAKE_SOURCE_DIR}/../../code2port/src/CalcManager/Ratpack")
set(CALCMGR_DIR "${CMAKE_SOURCE_DIR}/../../code2port/src/CalcManager")

# Collect all Ratpack .cpp files
file(GLOB RATPACK_SOURCES "${RATPACK_DIR}/*.cpp")

# Build Ratpack as a static library
add_library(ratpack STATIC ${RATPACK_SOURCES})
target_include_directories(ratpack PUBLIC
    ${RATPACK_DIR}
    ${CALCMGR_DIR}
)
target_compile_definitions(ratpack PRIVATE
    _CRT_SECURE_NO_WARNINGS
    NOMINMAX
)
# _Frees_ptr_opt_ is a SAL annotation that may not resolve without /analyze.
# Use /FI to force-include sal.h, and define _Frees_ptr_opt_ as empty fallback.
target_compile_definitions(ratpack PRIVATE "_Frees_ptr_opt_=")

# Build the test harness executable
add_executable(ratpack_harness ratpack_harness.cpp)
target_link_libraries(ratpack_harness PRIVATE ratpack)
target_include_directories(ratpack_harness PRIVATE
    ${RATPACK_DIR}
    ${CALCMGR_DIR}
)
target_compile_definitions(ratpack_harness PRIVATE
    _CRT_SECURE_NO_WARNINGS
    NOMINMAX
    "_Frees_ptr_opt_="
)
