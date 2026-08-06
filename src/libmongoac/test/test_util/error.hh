#pragma once

#include <catch2/catch_test_macros.hpp>
#include <mongoac/error.h>
#include <test_util/string.hh>

#define CHECK_MONGOAC_ERROR_CATEGORY(error, category)       \
   if (1) {                                                 \
      CHECK(::mongoac_error_category(error) == (category)); \
   } else                                                   \
      ((void)0)

#define REQUIRE_MONGOAC_ERROR_CATEGORY(error, category)       \
   if (1) {                                                   \
      REQUIRE(::mongoac_error_category(error) == (category)); \
   } else                                                     \
      ((void)0)

#define CHECK_MONGOAC_ERROR_CODE(error, code)                                      \
   if (1) {                                                                        \
      CAPTURE(::mongoac::test_util::to_string(::mongoac_error_message(error)));    \
      if (::mongoac_error_category(error) == MONGOAC_ERROR_CATEGORY_MONGOAC) {     \
         CHECK(::mongoac_error_code(error) == (code));                             \
      } else {                                                                     \
         CHECK(::mongoac_error_category(error) == MONGOAC_ERROR_CATEGORY_MONGOAC); \
      }                                                                            \
   } else                                                                          \
      ((void)0)

#define REQUIRE_MONGOAC_ERROR_CODE(error, code)                                      \
   if (1) {                                                                          \
      CAPTURE(::mongoac::test_util::to_string(::mongoac_error_message(error)));      \
      if (::mongoac_error_category(error) == MONGOAC_ERROR_CATEGORY_MONGOAC) {       \
         REQUIRE(::mongoac_error_code(error) == (code));                             \
      } else {                                                                       \
         REQUIRE(::mongoac_error_category(error) == MONGOAC_ERROR_CATEGORY_MONGOAC); \
      }                                                                              \
   } else                                                                            \
      ((void)0)

#define CHECK_MONGOAC_OK(error)                                                 \
   if (1) {                                                                     \
      CAPTURE(::mongoac::test_util::to_string(::mongoac_error_message(error))); \
      CHECK(::mongoac_error_code(error) == MONGOAC_ERROR_CODE_OK);              \
   } else                                                                       \
      ((void)0)

#define REQUIRE_MONGOAC_OK(error)                                               \
   if (1) {                                                                     \
      CAPTURE(::mongoac::test_util::to_string(::mongoac_error_message(error))); \
      REQUIRE(::mongoac_error_code(error) == MONGOAC_ERROR_CODE_OK);            \
   } else                                                                       \
      ((void)0)

#define CHECK_FALSE_MONGOAC_OK(error)                                           \
   if (1) {                                                                     \
      CAPTURE(::mongoac::test_util::to_string(::mongoac_error_message(error))); \
      CHECK_FALSE(::mongoac_error_code(error) == MONGOAC_ERROR_CODE_OK);        \
   } else                                                                       \
      ((void)0)

#define REQUIRE_FALSE_MONGOAC_OK(error)                                         \
   if (1) {                                                                     \
      CAPTURE(::mongoac::test_util::to_string(::mongoac_error_message(error))); \
      REQUIRE_FALSE(::mongoac_error_code(error) == MONGOAC_ERROR_CODE_OK);      \
   } else                                                                       \
      ((void)0)

#define CHECK_MONGOAC_INVALID_ARGUMENT(error) CHECK_MONGOAC_ERROR_CODE(error, MONGOAC_ERROR_CODE_INVALID_ARGUMENT)
#define REQUIRE_MONGOAC_INVALID_ARGUMENT(error) REQUIRE_MONGOAC_ERROR_CODE(error, MONGOAC_ERROR_CODE_INVALID_ARGUMENT)

#define CHECK_MONGOAC_RUNTIME_ERROR(error) CHECK_MONGOAC_ERROR_CODE(error, MONGOAC_ERROR_CODE_RUNTIME_ERROR)
#define REQUIRE_MONGOAC_RUNTIME_ERROR(error) REQUIRE_MONGOAC_ERROR_CODE(error, MONGOAC_ERROR_CODE_RUNTIME_ERROR)
