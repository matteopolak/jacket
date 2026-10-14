cmake_minimum_required(VERSION 3.21)
project(JaiSDL3 LANGUAGES C)
if(NOT SDL_SOURCE OR NOT SDL_JAI_PLATFORM)
  message(FATAL_ERROR "SDL_SOURCE and SDL_JAI_PLATFORM are required")
endif()
if(SDL_JAI_PLATFORM MATCHES "arm64$")
  set(JAI_SDL3_CPU JAI_SDL3_CPU_ARM64=1)
else()
  set(JAI_SDL3_CPU JAI_SDL3_CPU_X64=1)
endif()
add_subdirectory("${SDL_SOURCE}" sdl)
set_source_files_properties("${CMAKE_CURRENT_LIST_DIR}/inline-${SDL_JAI_PLATFORM}.c" "${CMAKE_CURRENT_LIST_DIR}/macros.c"
                            TARGET_DIRECTORY SDL3-static SDL3-shared PROPERTIES SKIP_PRECOMPILE_HEADERS ON)
# SDL's header-only helpers and real function-like macros, with stable C entry points.
# SDL3's own public export annotations put them in the runtime as well as the archive.
foreach(target SDL3-static SDL3-shared)
  target_sources(${target} PRIVATE "${CMAKE_CURRENT_LIST_DIR}/inline-${SDL_JAI_PLATFORM}.c" "${CMAKE_CURRENT_LIST_DIR}/macros.c")
endforeach()
if(MSVC)
  # Match the existing native pipeline: /MT plain CRT calls without naming a default CRT.
  # jaic executables can then link the archive with their selected runtime.
  target_compile_options(SDL3-static PRIVATE /Zl)
endif()
file(GENERATE OUTPUT "${CMAKE_BINARY_DIR}/static-link-libraries-$<CONFIG>.txt"
     CONTENT "$<TARGET_PROPERTY:SDL3-static,INTERFACE_LINK_LIBRARIES>\n$<TARGET_PROPERTY:SDL3-static,INTERFACE_LINK_OPTIONS>\n")

# Extend SDL's official ELF version script in the build directory, preserving the source.
if(UNIX AND NOT APPLE)
  file(READ "${SDL_SOURCE}/src/dynapi/SDL_dynapi.sym" symbols)
  string(REPLACE "# extra symbols go here" "JAI_SDL3_*; # extra symbols go here" symbols "${symbols}")
  file(WRITE "${CMAKE_BINARY_DIR}/jai-sdl3.sym" "${symbols}")
  get_target_property(link_options SDL3-shared LINK_OPTIONS)
  string(REPLACE "${SDL_SOURCE}/src/dynapi/SDL_dynapi.sym" "${CMAKE_BINARY_DIR}/jai-sdl3.sym" link_options "${link_options}")
  set_property(TARGET SDL3-shared PROPERTY LINK_OPTIONS "${link_options}")
endif()
# Build both linkage modes, with SDL supplying every transitive platform dependency.
foreach(mode static shared)
  add_executable(jai-sdl3-smoke-${mode} smoke.c)
  target_compile_definitions(jai-sdl3-smoke-${mode} PRIVATE ${JAI_SDL3_CPU}
    JAI_SDL3_LAYOUT_HEADER="${CMAKE_CURRENT_LIST_DIR}/../../../tests/sdl3/c-layout-${SDL_JAI_PLATFORM}.h")
  set_target_properties(jai-sdl3-smoke-${mode} PROPERTIES C_STANDARD 11 C_STANDARD_REQUIRED YES)
  target_link_libraries(jai-sdl3-smoke-${mode} PRIVATE SDL3-${mode})
endforeach()
if(WIN32)
  add_custom_command(TARGET jai-sdl3-smoke-shared POST_BUILD
    COMMAND ${CMAKE_COMMAND} -E copy_if_different "$<TARGET_FILE:SDL3-shared>" "$<TARGET_FILE_DIR:jai-sdl3-smoke-shared>")
endif()
