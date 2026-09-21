include (FetchCatch2)

if (NOT BUILD_TESTING)
    set_property (DIRECTORY PROPERTY EXCLUDE_FROM_ALL TRUE)
endif ()

if (TARGET mongoac_shared)
  set (mongoac_target mongoac_shared)
else ()
  set (mongoac_target mongoac_static)
endif ()

if (TARGET bson::shared)
  set (bson_target bson::shared)
else ()
  set (bson_target bson::static)
endif ()

add_executable (test-libmongoac)
target_compile_features (test-libmongoac PRIVATE cxx_std_17)
target_link_libraries (test-libmongoac PRIVATE "${mongoac_target}" Catch2::Catch2WithMain)

# Suppress compiler warnings emitted by bson headers.
target_link_libraries (test-libmongoac PRIVATE "${mongoac_target}" "$<LINK_ONLY:${bson_target}>" Catch2::Catch2WithMain)
target_include_directories (test-libmongoac
  SYSTEM PRIVATE "$<TARGET_PROPERTY:${bson_target},INTERFACE_INCLUDE_DIRECTORIES>"
)

mongo_bool_setting(
  ENABLE_MONGOAC_EXAMPLES
  "Enable building MongoDB Async C examples (as tests)"
)
if (ENABLE_MONGOAC_EXAMPLES)
  include (FetchLibuv)
  target_link_libraries (test-libmongoac PRIVATE libuv::libuv)
endif ()

file (GLOB_RECURSE mongoac_test_sources CONFIGURE_DEPENDS "${CMAKE_CURRENT_SOURCE_DIR}/*.cpp")
file (GLOB_RECURSE mongoac_test_headers CONFIGURE_DEPENDS "${CMAKE_CURRENT_SOURCE_DIR}/*.hh")
target_sources (test-libmongoac
  PRIVATE
    ${mongoac_test_sources}
  PRIVATE
    FILE_SET HEADERS
      BASE_DIRS "${CMAKE_CURRENT_SOURCE_DIR}"
      FILES ${mongoac_test_headers}
)
FetchContent_GetProperties (EP_Catch2)
list (APPEND CMAKE_MODULE_PATH "${ep_catch2_SOURCE_DIR}/extras")
include (Catch)
include (CTest)

catch_discover_tests (test-libmongoac
  TEST_SPEC "[run-serial]"
  TEST_PREFIX "mongoac/"
  ADD_TAGS_AS_LABELS
  PROPERTIES
    RUN_SERIAL TRUE
)

catch_discover_tests (test-libmongoac
  TEST_SPEC "~[run-serial]"
  TEST_PREFIX "mongoac/"
  ADD_TAGS_AS_LABELS
)
