#include <mongoac/client_options.h>

#include <bson/bson.h>

#include <catch2/catch_test_macros.hpp>
#include <catch2/matchers/catch_matchers_string.hpp>
#include <mongoac/error.h>
#include <test_util/owning_ptr.hpp>

#include <cstring>

using mongoac::test_util::make_owning_ptr;

TEST_CASE("new", "[mongoac][client_options]")
{
   SECTION("default")
   {
      auto const opts = make_owning_ptr(mongoac_client_options_new(), &mongoac_client_options_destroy);
      REQUIRE(opts != nullptr);
   }
}

TEST_CASE("destroy", "[mongoac][client_options]")
{
   SECTION("null")
   {
      mongoac_client_options_destroy(nullptr);
      SUCCEED();
   }
}

TEST_CASE("set_capture_command_events", "[mongoac][client_options]")
{
   auto const opts = make_owning_ptr(mongoac_client_options_new(), &mongoac_client_options_destroy);
   REQUIRE(opts != nullptr);

   SECTION("null")
   {
      mongoac_client_options_set_capture_command_events(nullptr, true);
      SUCCEED();
   }

   SECTION("set true")
   {
      mongoac_client_options_set_capture_command_events(opts, true);
      SUCCEED();
   }

   SECTION("set false")
   {
      mongoac_client_options_set_capture_command_events(opts, false);
      SUCCEED();
   }
}

static bson_t *
build_server_api(const char *version, bool set_strict, bool strict_val, bool set_deprecation, bool deprecation_val)
{
   return BCON_NEW("apiVersion",
                   BCON_UTF8(version),
                   "apiStrict",
                   BCON_BOOL(set_strict ? strict_val : false),
                   "apiDeprecationErrors",
                   BCON_BOOL(set_deprecation ? deprecation_val : false));
}

TEST_CASE("set_server_api", "[mongoac][client_options]")
{
   auto const opts = make_owning_ptr(mongoac_client_options_new(), &mongoac_client_options_destroy);
   REQUIRE(opts != nullptr);

   SECTION("null")
   {
      auto const error = make_owning_ptr(mongoac_error_new(), &mongoac_error_destroy);
      auto const bson = make_owning_ptr(build_server_api("1", false, false, false, false), &bson_destroy);

      mongoac_client_options_set_server_api(nullptr, bson, error);
      CHECK(mongoac_error_category(error) == MONGOAC_ERROR_CATEGORY_MONGOAC);
      CHECK(mongoac_error_code(error) == MONGOAC_ERROR_CODE_INVALID_ARGUMENT);
   }

   SECTION("null ")
   {
      auto const error = make_owning_ptr(mongoac_error_new(), &mongoac_error_destroy);

      mongoac_client_options_set_server_api(opts, nullptr, error);
      CHECK(mongoac_error_category(error) == MONGOAC_ERROR_CATEGORY_NONE);
   }

   SECTION("valid")
   {
      auto const bson = make_owning_ptr(build_server_api("1", false, false, false, false), &bson_destroy);

      mongoac_client_options_set_server_api(opts, bson, nullptr);
   }

   SECTION("strict")
   {
      auto const bson = make_owning_ptr(build_server_api("1", true, true, false, false), &bson_destroy);

      mongoac_client_options_set_server_api(opts, bson, nullptr);
   }

   SECTION("all valid")
   {
      auto const bson = make_owning_ptr(build_server_api("1", true, true, true, true), &bson_destroy);

      mongoac_client_options_set_server_api(opts, bson, nullptr);
   }

   SECTION("invalid apiVersion")
   {
      auto const error = make_owning_ptr(mongoac_error_new(), &mongoac_error_destroy);
      auto const bson = make_owning_ptr(build_server_api("2", false, false, false, false), &bson_destroy);

      mongoac_client_options_set_server_api(opts, bson, error);
   }

   SECTION("invalid")
   {
      auto const error = make_owning_ptr(mongoac_error_new(), &mongoac_error_destroy);
      bson_t bson = BSON_INITIALIZER;

      mongoac_client_options_set_server_api(opts, &bson, error);

      bson_destroy(&bson);
   }

   SECTION("wrong type for apiVersion")
   {
      auto const error = make_owning_ptr(mongoac_error_new(), &mongoac_error_destroy);
      auto const bson = make_owning_ptr(BCON_NEW("apiVersion", BCON_BOOL(true)), &bson_destroy);

      mongoac_client_options_set_server_api(opts, bson, error);
   }
}
