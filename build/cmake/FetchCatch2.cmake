# Use FetchContent to obtain Catch2.

include(FetchContent)

function(fetch_catch2)
    FetchContent_Declare(
        EP_Catch2

        GIT_REPOSITORY https://github.com/catchorg/Catch2
        GIT_TAG v3.15.3
        GIT_SHALLOW TRUE
        GIT_REMOTE_UPDATE_STRATEGY CHECKOUT
        LOG_DOWNLOAD ON

        # Support registering Catch2 tests with CTest uniquely by tags.
        PATCH_COMMAND git apply "${CMAKE_SOURCE_DIR}/build/cmake/catch-add-tests-with-tags.patch"

        SYSTEM
    )

    FetchContent_GetProperties(EP_Catch2)

    if(NOT ep_catch2_POPULATED)
        message(STATUS "Downloading Catch2...")

        # Avoid Catch2 compile warnings from being treated as errors.
        string(REPLACE " -Werror" "" CMAKE_C_FLAGS "${CMAKE_C_FLAGS}")
        string(REPLACE " -Werror" "" CMAKE_CXX_FLAGS "${CMAKE_CXX_FLAGS}")

        FetchContent_MakeAvailable(EP_Catch2)

        # Avoid building unnecessary targets. Use FetchContent_Declare(EXCLUDE_FROM_ALL) in CMake 3.28 and newer.
        set_property(DIRECTORY "${ep_catch2_SOURCE_DIR}" PROPERTY EXCLUDE_FROM_ALL ON)

        # Catch2 config vars.
        set_property(CACHE CATCH_CONFIG_CPP11_TO_STRING PROPERTY VALUE ON)
        set_property(CACHE CATCH_CONFIG_CPP17_OPTIONAL PROPERTY VALUE ON)
        set_property(CACHE CATCH_CONFIG_CPP17_STRING_VIEW PROPERTY VALUE ON)
        set_property(CACHE CATCH_INSTALL_DOCS PROPERTY VALUE OFF)
        set_property(CACHE CATCH_INSTALL_EXTRAS PROPERTY VALUE OFF)

        message (STATUS "Downloading Catch2... done.")
    endif()

    if (ep_catch2_SOURCE_DIR)
        set (catch_add_tests_file "${ep_catch2_SOURCE_DIR}/extras/CatchAddTests.cmake")
        if (EXISTS "${catch_add_tests_file}")
            file (READ "${catch_add_tests_file}" catch_add_tests_file_content)
            if (NOT catch_add_tests_file_content MATCHES "catch2-add-tests-with-tags")
                message (
                    WARNING
                    "Unpatched Catch2 library (at ${ep_catch2_SOURCE_DIR}) may not register test cases with CTest"
                    "correctly: some tests may be skipped!"
                )
            endif ()
        endif ()
    endif ()
endfunction()

fetch_catch2()
