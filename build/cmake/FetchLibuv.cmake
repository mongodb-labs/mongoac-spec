# Use FetchContent to obtain libuv.

include(FetchContent)

function(fetch_libuv)
    FetchContent_Declare(
        EP_libuv

        GIT_REPOSITORY https://github.com/libuv/libuv.git
        GIT_TAG v1.52.1
        GIT_SHALLOW TRUE
        GIT_REMOTE_UPDATE_STRATEGY CHECKOUT
        LOG_DOWNLOAD ON

        SYSTEM
    )

    FetchContent_GetProperties(EP_libuv)

    if(NOT ep_libuv_POPULATED)
        message(STATUS "Downloading libuv...")

        # Avoid libuv compile warnings from being treated as errors.
        string(REPLACE " -Werror" "" CMAKE_C_FLAGS "${CMAKE_C_FLAGS}")
        string(REPLACE " -Werror" "" CMAKE_CXX_FLAGS "${CMAKE_CXX_FLAGS}")

        FetchContent_MakeAvailable(EP_libuv)

        # Avoid building unnecessary targets. Use FetchContent_Declare(EXCLUDE_FROM_ALL) in CMake 3.28 and newer.
        set_property(DIRECTORY "${ep_libuv_SOURCE_DIR}" PROPERTY EXCLUDE_FROM_ALL ON)

        message (STATUS "Downloading libuv... done.")
    endif()
endfunction()

fetch_libuv()
