#include <mongoac/future.h>

//

#include <catch2/catch_test_macros.hpp>
#include <mongoac/bson.h>
#include <mongoac/client_session.h>
#include <mongoac/error.h>
#include <test_util/owning_ptr.hh>

using mongoac::test_util::make_owning_ptr;

TEST_CASE("clone", "[mongoac][future]")
{
   SECTION("null")
   {
      CHECK(mongoac_future_clone(nullptr) == nullptr);
   }
}

TEST_CASE("destroy", "[mongoac][future]")
{
   SECTION("null")
   {
      mongoac_future_destroy(nullptr);
      SUCCEED();
   }
}

TEST_CASE("is_ready", "[mongoac][future]")
{
   SECTION("null")
   {
      CHECK(mongoac_future_is_ready(nullptr) == false);
   }
}

TEST_CASE("poll", "[mongoac][future]")
{
   SECTION("null")
   {
      CHECK(mongoac_future_poll(nullptr) == false);
   }
}

TEST_CASE("get_void", "[mongoac][future]")
{
   SECTION("null future")
   {
      auto const error = make_owning_ptr(mongoac_error_new(), &mongoac_error_destroy);
      mongoac_future_get_void(nullptr, error);
      CHECK(mongoac_error_code(error) == MONGOAC_ERROR_CODE_INVALID_ARGUMENT);
   }
}

TEST_CASE("get_int32", "[mongoac][future]")
{
   SECTION("null future")
   {
      auto const error = make_owning_ptr(mongoac_error_new(), &mongoac_error_destroy);
      CHECK(mongoac_future_get_int32(nullptr, error) == 0);
      CHECK(mongoac_error_code(error) == MONGOAC_ERROR_CODE_INVALID_ARGUMENT);
   }
}

TEST_CASE("get_bson", "[mongoac][future]")
{
   SECTION("null future")
   {
      auto const error = make_owning_ptr(mongoac_error_new(), &mongoac_error_destroy);
      CHECK(mongoac_future_get_bson(nullptr, error).data == nullptr);
      CHECK(mongoac_error_code(error) == MONGOAC_ERROR_CODE_INVALID_ARGUMENT);
   }
}

TEST_CASE("get_client_session", "[mongoac][future]")
{
   SECTION("null future")
   {
      auto const error = make_owning_ptr(mongoac_error_new(), &mongoac_error_destroy);
      CHECK(mongoac_future_get_client_session(nullptr, error) == nullptr);
      CHECK(mongoac_error_code(error) == MONGOAC_ERROR_CODE_INVALID_ARGUMENT);
   }
}
